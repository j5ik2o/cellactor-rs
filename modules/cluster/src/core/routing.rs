//! Routing utilities for cluster requests.

/// Identity→PID cache to reduce lookup pressure.
pub mod pid_cache;

pub use pid_cache::{PidCache, PidCacheEntry};
