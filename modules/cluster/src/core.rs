//! Core primitives for virtual-actor identity lookup.

/// Configuration structures for the cluster runtime.
pub mod config;
/// Runtime container coordinating identity lookup and activation ledger.
pub mod runtime;
/// Activation ledger data structures.
pub mod activation;
/// Identity lookup services and rendezvous logic.
pub mod identity;
/// Metrics abstractions and sinks.
pub mod metrics;
/// Routing helpers such as PID caches.
pub mod routing;
/// Placement coordination and activation drivers.
pub mod placement;
