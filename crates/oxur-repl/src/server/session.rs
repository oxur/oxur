//! Session Management for REPL Server
//!
//! Manages multiple concurrent REPL sessions, each with its own evaluation context.
//! Provides session lifecycle management: create, clone, lookup, close.
//!
//! Based on ODD-0018: Oxur Remote REPL Protocol Design

use crate::eval::{EvalContext, EvalError};
use crate::protocol::{ReplMode, SessionId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Session manager errors
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Session not found: {0}")]
    NotFound(String),

    #[error("Session already exists: {0}")]
    AlreadyExists(String),

    #[error("Lock poisoned")]
    LockPoisoned,

    #[error("Evaluation failed")]
    EvalFailed(#[from] EvalError),
}

pub type Result<T> = std::result::Result<T, SessionError>;

/// Session information
#[derive(Debug, Clone)]
pub struct SessionInfo {
    /// Session ID
    pub id: SessionId,

    /// Optional session name
    pub name: Option<String>,

    /// Evaluation mode (Lisp or Sexpr)
    pub mode: ReplMode,

    /// Number of evaluations performed
    pub eval_count: u64,

    /// Creation timestamp (milliseconds since epoch)
    pub created_at: u64,

    /// Last active timestamp (milliseconds since epoch)
    pub last_active_at: u64,

    /// Session timeout in milliseconds (default: 1 hour)
    pub timeout_ms: u64,
}

/// Session metadata tracked separately from EvalContext
#[derive(Debug, Clone)]
struct SessionMetadata {
    /// Optional session name
    name: Option<String>,
    /// Creation timestamp (milliseconds since epoch)
    created_at: u64,
    /// Last active timestamp (milliseconds since epoch)
    last_active_at: u64,
    /// Session timeout in milliseconds
    timeout_ms: u64,
}

impl Default for SessionMetadata {
    fn default() -> Self {
        Self {
            name: None,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            last_active_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            timeout_ms: 3_600_000, // Default: 1 hour
        }
    }
}

/// Manages multiple REPL sessions
///
/// Thread-safe session storage using Arc<RwLock<HashMap>>.
/// Each session has its own EvalContext with isolated state.
#[derive(Debug, Clone)]
pub struct SessionManager {
    /// Active sessions (session_id -> context)
    sessions: Arc<RwLock<HashMap<SessionId, EvalContext>>>,
    /// Session metadata (session_id -> metadata)
    metadata: Arc<RwLock<HashMap<SessionId, SessionMetadata>>>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new session
    ///
    /// # Errors
    ///
    /// Returns `SessionError::AlreadyExists` if a session with the same ID already exists.
    ///
    /// # Example
    ///
    /// ```
    /// use oxur_repl::server::SessionManager;
    /// use oxur_repl::protocol::{ReplMode, SessionId};
    ///
    /// let manager = SessionManager::new();
    /// let session_id = manager.create(SessionId::new("session-1"), ReplMode::Lisp).unwrap();
    /// assert_eq!(session_id, SessionId::new("session-1"));
    /// ```
    pub fn create(&self, session_id: SessionId, mode: ReplMode) -> Result<SessionId> {
        let mut sessions = self.sessions.write().map_err(|_| SessionError::LockPoisoned)?;

        if sessions.contains_key(&session_id) {
            return Err(SessionError::AlreadyExists(session_id.to_string()));
        }

        // Use with_compilation to get full session directory and artifact cache support
        let context = EvalContext::with_compilation(session_id.clone(), mode)?;
        sessions.insert(session_id.clone(), context);

        // Insert default metadata
        let mut metadata = self.metadata.write().map_err(|_| SessionError::LockPoisoned)?;
        metadata.insert(session_id.clone(), SessionMetadata::default());

        Ok(session_id)
    }

    /// Execute an async operation with a session
    ///
    /// Takes ownership of the evaluation, executes it, and updates the session.
    /// This avoids lifetime issues with async closures.
    ///
    /// # Errors
    ///
    /// Returns `SessionError::NotFound` if the session doesn't exist.
    /// Returns `SessionError::EvalFailed` if evaluation fails due to:
    /// - Syntax errors in the code
    /// - Type errors during compilation
    /// - Runtime errors during execution
    /// - Compilation errors
    /// - Unsupported operations for the current tier
    ///
    /// Returns `SessionError::LockPoisoned` if the internal lock is poisoned.
    pub async fn eval(
        &self,
        session_id: &SessionId,
        code: impl AsRef<str>,
    ) -> Result<crate::eval::EvalResult> {
        // First, get a clone of the context to evaluate with
        let mut context = {
            let sessions = self.sessions.read().map_err(|_| SessionError::LockPoisoned)?;

            sessions
                .get(session_id)
                .ok_or_else(|| SessionError::NotFound(session_id.to_string()))?
                .clone()
        };

        // Evaluate the code
        let result = context.eval(code.as_ref()).await?;

        // Update the session with the new state
        {
            let mut sessions = self.sessions.write().map_err(|_| SessionError::LockPoisoned)?;

            sessions.insert(session_id.clone(), context.clone());
        }

        // Update last_active_at timestamp
        {
            let mut metadata = self.metadata.write().map_err(|_| SessionError::LockPoisoned)?;
            if let Some(meta) = metadata.get_mut(session_id) {
                meta.last_active_at = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
            }
        }

        Ok(result)
    }

    /// Clone an existing session
    ///
    /// Creates a new session with the same cache and state as the source session,
    /// but with a different session ID and reset statistics.
    ///
    /// # Errors
    ///
    /// Returns `SessionError::NotFound` if the source session doesn't exist.
    /// Returns `SessionError::AlreadyExists` if the target session ID is already in use.
    pub fn clone_session(&self, source_id: &SessionId, target_id: SessionId) -> Result<SessionId> {
        let mut sessions = self.sessions.write().map_err(|_| SessionError::LockPoisoned)?;

        let source_context =
            sessions.get(source_id).ok_or_else(|| SessionError::NotFound(source_id.to_string()))?;

        if sessions.contains_key(&target_id) {
            return Err(SessionError::AlreadyExists(target_id.to_string()));
        }

        let cloned_context = source_context.clone_to(target_id.clone());
        sessions.insert(target_id.clone(), cloned_context);

        Ok(target_id)
    }

    /// Close a session
    ///
    /// Removes the session and frees its resources.
    ///
    /// # Errors
    ///
    /// Returns `SessionError::NotFound` if the session doesn't exist.
    pub fn close(&self, session_id: &SessionId) -> Result<()> {
        let mut sessions = self.sessions.write().map_err(|_| SessionError::LockPoisoned)?;

        sessions
            .remove(session_id)
            .ok_or_else(|| SessionError::NotFound(session_id.to_string()))?;

        Ok(())
    }

    /// List all active sessions
    ///
    /// Returns information about all currently active sessions.
    ///
    /// # Errors
    ///
    /// Returns `SessionError::LockPoisoned` if the internal lock is poisoned.
    pub fn list(&self) -> Result<Vec<SessionInfo>> {
        let sessions = self.sessions.read().map_err(|_| SessionError::LockPoisoned)?;
        let metadata = self.metadata.read().map_err(|_| SessionError::LockPoisoned)?;

        let mut infos: Vec<SessionInfo> = sessions
            .iter()
            .map(|(id, ctx)| {
                let (tier1, tier2, _) = ctx.stats();
                let meta = metadata.get(id).cloned().unwrap_or_default();

                SessionInfo {
                    id: id.clone(),
                    name: meta.name,
                    mode: ctx.mode(),
                    eval_count: tier1 + tier2,
                    created_at: meta.created_at,
                    last_active_at: meta.last_active_at,
                    timeout_ms: meta.timeout_ms,
                }
            })
            .collect();

        // Sort by session ID for consistent ordering
        infos.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(infos)
    }

    /// Get information about a specific session
    ///
    /// # Errors
    ///
    /// Returns `SessionError::NotFound` if the session doesn't exist.
    pub fn get_info(&self, session_id: &SessionId) -> Result<SessionInfo> {
        let sessions = self.sessions.read().map_err(|_| SessionError::LockPoisoned)?;
        let metadata = self.metadata.read().map_err(|_| SessionError::LockPoisoned)?;

        let ctx = sessions
            .get(session_id)
            .ok_or_else(|| SessionError::NotFound(session_id.to_string()))?;

        let (tier1, tier2, _) = ctx.stats();
        let meta = metadata.get(session_id).cloned().unwrap_or_default();

        Ok(SessionInfo {
            id: session_id.clone(),
            name: meta.name,
            mode: ctx.mode(),
            eval_count: tier1 + tier2,
            created_at: meta.created_at,
            last_active_at: meta.last_active_at,
            timeout_ms: meta.timeout_ms,
        })
    }

    /// Get the statistics collector for a session
    ///
    /// Returns a shared reference to the session's StatsCollector for detailed metrics access.
    ///
    /// # Errors
    ///
    /// Returns `SessionError::NotFound` if the session doesn't exist.
    /// Returns `SessionError::LockPoisoned` if the internal lock is poisoned.
    pub fn get_stats_collector(
        &self,
        session_id: &SessionId,
    ) -> Result<Arc<std::sync::Mutex<crate::eval::EvalMetrics>>> {
        let sessions = self.sessions.read().map_err(|_| SessionError::LockPoisoned)?;

        let ctx = sessions
            .get(session_id)
            .ok_or_else(|| SessionError::NotFound(session_id.to_string()))?;

        Ok(ctx.stats_collector())
    }

    /// Get resource statistics for a session
    ///
    /// Returns (session_dir_stats, artifact_cache_stats) for the given session.
    ///
    /// # Errors
    ///
    /// Returns `SessionError::NotFound` if the session doesn't exist.
    /// Returns `SessionError::LockPoisoned` if the internal lock is poisoned.
    pub fn get_resource_stats(
        &self,
        session_id: &SessionId,
    ) -> Result<(Option<crate::session::DirStats>, Option<crate::cache::CacheStats>)> {
        let sessions = self.sessions.read().map_err(|_| SessionError::LockPoisoned)?;

        let ctx = sessions
            .get(session_id)
            .ok_or_else(|| SessionError::NotFound(session_id.to_string()))?;

        Ok((ctx.session_dir_stats(), ctx.artifact_cache_stats()))
    }

    /// Get subprocess statistics for a session
    ///
    /// Returns subprocess metrics snapshot for the given session.
    ///
    /// # Errors
    ///
    /// Returns `SessionError::NotFound` if the session doesn't exist.
    /// Returns `SessionError::LockPoisoned` if the internal lock is poisoned.
    pub fn get_subprocess_stats(
        &self,
        session_id: &SessionId,
    ) -> Result<Option<crate::metrics::SubprocessMetricsSnapshot>> {
        let sessions = self.sessions.read().map_err(|_| SessionError::LockPoisoned)?;

        let ctx = sessions
            .get(session_id)
            .ok_or_else(|| SessionError::NotFound(session_id.to_string()))?;

        Ok(ctx.subprocess_stats())
    }

    /// Get the number of active sessions
    ///
    /// # Errors
    ///
    /// Returns `SessionError::LockPoisoned` if the internal lock is poisoned.
    pub fn count(&self) -> Result<usize> {
        let sessions = self.sessions.read().map_err(|_| SessionError::LockPoisoned)?;

        Ok(sessions.len())
    }

    /// Check if a session exists
    ///
    /// # Errors
    ///
    /// Returns `SessionError::LockPoisoned` if the internal lock is poisoned.
    pub fn exists(&self, session_id: &SessionId) -> Result<bool> {
        let sessions = self.sessions.read().map_err(|_| SessionError::LockPoisoned)?;

        Ok(sessions.contains_key(session_id))
    }

    /// Close all sessions
    ///
    /// Useful for server shutdown. Returns the number of sessions that were closed.
    ///
    /// # Errors
    ///
    /// Returns `SessionError::LockPoisoned` if the internal lock is poisoned.
    pub fn close_all(&self) -> Result<usize> {
        let mut sessions = self.sessions.write().map_err(|_| SessionError::LockPoisoned)?;

        let count = sessions.len();
        sessions.clear();

        Ok(count)
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session() {
        let manager = SessionManager::new();
        let id = manager.create(SessionId::new("test-1"), ReplMode::Lisp).unwrap();

        assert_eq!(id, SessionId::new("test-1"));
        assert!(manager.exists(&id).unwrap());
    }

    #[test]
    fn test_create_duplicate_session() {
        let manager = SessionManager::new();
        manager.create(SessionId::new("test-1"), ReplMode::Lisp).unwrap();

        let result = manager.create(SessionId::new("test-1"), ReplMode::Lisp);
        assert!(matches!(result, Err(SessionError::AlreadyExists(_))));
    }

    #[tokio::test]
    async fn test_eval() {
        let manager = SessionManager::new();
        manager.create(SessionId::new("test"), ReplMode::Lisp).unwrap();

        let result = manager.eval(&SessionId::new("test"), "(+ 1 2)").await.unwrap();

        assert_eq!(result.value, "3");
    }

    #[tokio::test]
    async fn test_eval_not_found() {
        let manager = SessionManager::new();

        let result = manager.eval(&SessionId::new("nonexistent"), "test").await;

        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[test]
    fn test_clone_session() {
        let manager = SessionManager::new();
        manager.create(SessionId::new("source"), ReplMode::Lisp).unwrap();

        let cloned_id =
            manager.clone_session(&SessionId::new("source"), SessionId::new("target")).unwrap();

        assert_eq!(cloned_id, SessionId::new("target"));
        assert!(manager.exists(&SessionId::new("target")).unwrap());
    }

    #[test]
    fn test_clone_nonexistent_session() {
        let manager = SessionManager::new();

        let result =
            manager.clone_session(&SessionId::new("nonexistent"), SessionId::new("target"));

        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[test]
    fn test_clone_to_existing_session() {
        let manager = SessionManager::new();
        manager.create(SessionId::new("source"), ReplMode::Lisp).unwrap();
        manager.create(SessionId::new("target"), ReplMode::Lisp).unwrap();

        let result = manager.clone_session(&SessionId::new("source"), SessionId::new("target"));

        assert!(matches!(result, Err(SessionError::AlreadyExists(_))));
    }

    #[test]
    fn test_close_session() {
        let manager = SessionManager::new();
        manager.create(SessionId::new("test"), ReplMode::Lisp).unwrap();

        assert!(manager.exists(&SessionId::new("test")).unwrap());

        manager.close(&SessionId::new("test")).unwrap();

        assert!(!manager.exists(&SessionId::new("test")).unwrap());
    }

    #[test]
    fn test_close_nonexistent_session() {
        let manager = SessionManager::new();

        let result = manager.close(&SessionId::new("nonexistent"));

        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[test]
    fn test_list_sessions() {
        let manager = SessionManager::new();
        manager.create(SessionId::new("session-1"), ReplMode::Lisp).unwrap();
        manager.create(SessionId::new("session-2"), ReplMode::Sexpr).unwrap();

        let sessions = manager.list().unwrap();

        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].id, SessionId::new("session-1"));
        assert_eq!(sessions[0].mode, ReplMode::Lisp);
        assert_eq!(sessions[1].id, SessionId::new("session-2"));
        assert_eq!(sessions[1].mode, ReplMode::Sexpr);
    }

    #[test]
    fn test_list_empty() {
        let manager = SessionManager::new();
        let sessions = manager.list().unwrap();

        assert_eq!(sessions.len(), 0);
    }

    #[test]
    fn test_get_info() {
        let manager = SessionManager::new();
        manager.create(SessionId::new("test"), ReplMode::Lisp).unwrap();

        let info = manager.get_info(&SessionId::new("test")).unwrap();

        assert_eq!(info.id, SessionId::new("test"));
        assert_eq!(info.mode, ReplMode::Lisp);
        assert_eq!(info.eval_count, 0);
    }

    #[test]
    fn test_get_info_nonexistent() {
        let manager = SessionManager::new();

        let result = manager.get_info(&SessionId::new("nonexistent"));

        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_eval_count_tracking() {
        let manager = SessionManager::new();
        manager.create(SessionId::new("test"), ReplMode::Lisp).unwrap();

        // Perform some evaluations
        manager.eval(&SessionId::new("test"), "(+ 1 2)").await.unwrap();

        manager.eval(&SessionId::new("test"), "(* 3 4)").await.unwrap();

        let info = manager.get_info(&SessionId::new("test")).unwrap();
        assert_eq!(info.eval_count, 2);
    }

    #[test]
    fn test_count() {
        let manager = SessionManager::new();

        assert_eq!(manager.count().unwrap(), 0);

        manager.create(SessionId::new("session-1"), ReplMode::Lisp).unwrap();
        assert_eq!(manager.count().unwrap(), 1);

        manager.create(SessionId::new("session-2"), ReplMode::Sexpr).unwrap();
        assert_eq!(manager.count().unwrap(), 2);

        manager.close(&SessionId::new("session-1")).unwrap();
        assert_eq!(manager.count().unwrap(), 1);
    }

    #[test]
    fn test_close_all() {
        let manager = SessionManager::new();
        manager.create(SessionId::new("session-1"), ReplMode::Lisp).unwrap();
        manager.create(SessionId::new("session-2"), ReplMode::Sexpr).unwrap();
        manager.create(SessionId::new("session-3"), ReplMode::Lisp).unwrap();

        assert_eq!(manager.count().unwrap(), 3);

        let closed = manager.close_all().unwrap();

        assert_eq!(closed, 3);
        assert_eq!(manager.count().unwrap(), 0);
    }

    #[test]
    fn test_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let manager = Arc::new(SessionManager::new());
        let mut handles = vec![];

        // Create sessions from multiple threads
        for i in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let handle = thread::spawn(move || {
                manager_clone
                    .create(SessionId::new(format!("session-{}", i)), ReplMode::Lisp)
                    .unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(manager.count().unwrap(), 10);
    }
}
