//! Resource statistics for REPL evaluation
//!
//! Provides process-level resource statistics (memory, PID).
//! Evaluation metrics (timing, cache) are now in `crate::metrics::EvalMetrics`.

use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};

/// Resource usage statistics
#[derive(Debug, Clone)]
pub struct ResourceStats {
    /// Process memory (RSS - Resident Set Size) in bytes
    pub process_memory_bytes: u64,

    /// Virtual memory size in bytes
    pub virtual_memory_bytes: u64,

    /// Process ID
    pub pid: u32,
}

/// Get current process resource usage
pub fn get_resource_stats() -> Option<ResourceStats> {
    let pid = std::process::id();
    let mut system = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::new().with_memory()),
    );

    system.refresh_processes_specifics(ProcessRefreshKind::new().with_memory());

    let sysinfo_pid = Pid::from_u32(pid);
    let process = system.process(sysinfo_pid)?;

    Some(ResourceStats {
        process_memory_bytes: process.memory(),
        virtual_memory_bytes: process.virtual_memory(),
        pid,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_resource_stats() {
        // This test may fail in some CI environments without process info access
        let stats = get_resource_stats();
        if let Some(stats) = stats {
            assert!(stats.pid > 0);
            // Memory values should be non-zero for running process
            assert!(stats.process_memory_bytes > 0 || stats.virtual_memory_bytes > 0);
        }
    }
}
