//! Bootstrap utilities for integrating the cluster extension into a host ActorSystem.

/// Shared state of the cluster bootstrap lifecycle.
pub mod bootstrap_state;
/// Error type for persisting or retrieving bootstrap status.
pub mod bootstrap_status_error;
/// Trait defining the contract for bootstrap status persistence.
pub mod bootstrap_status_store;
/// Cluster bootstrap orchestration entrypoint.
pub mod cluster_bootstrap;
/// Configuration for cluster bootstrap.
pub mod cluster_bootstrap_config;
/// Errors returned from the bootstrap orchestration.
pub mod cluster_bootstrap_error;
/// Handle bundle returned after bootstrap.
pub mod cluster_extension_handle;
/// In-memory implementation of the bootstrap status store.
pub mod in_memory_bootstrap_status_store;

pub use bootstrap_state::BootstrapState;
pub use bootstrap_status_error::BootstrapStatusError;
pub use bootstrap_status_store::BootstrapStatusStore;
pub use cluster_bootstrap::ClusterBootstrap;
pub use cluster_bootstrap_config::ClusterBootstrapConfig;
pub use cluster_bootstrap_error::ClusterBootstrapError;
pub use cluster_extension_handle::ClusterExtensionHandle;
pub use in_memory_bootstrap_status_store::InMemoryBootstrapStatusStore;
