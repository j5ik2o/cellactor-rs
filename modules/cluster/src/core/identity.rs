//! Identity lookup and rendezvous hashing services.

/// Cluster identity descriptor shared across services.
pub mod cluster_identity;
/// Cluster node metadata used by rendezvous hashing.
pub mod cluster_node;
/// Hash ring provider that selects owners deterministically.
pub mod hash_ring_provider;
/// Core identity lookup service implementation placeholder.
pub mod identity_lookup_service;
/// Unique node identifier newtype.
pub mod node_id;
/// Snapshot of current topology and members.
pub mod topology_snapshot;

pub use cluster_identity::ClusterIdentity;
pub use cluster_node::ClusterNode;
pub use hash_ring_provider::HashRingProvider;
pub use identity_lookup_service::IdentityLookupService;
pub use node_id::NodeId;
pub use topology_snapshot::TopologySnapshot;
