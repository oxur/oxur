//! Caching infrastructure for compiled artifacts
//!
//! Provides persistent disk-based caching of compiled dynamic libraries
//! using SHA256 content-addressed storage.

mod artifact;

pub use artifact::ArtifactCache;
