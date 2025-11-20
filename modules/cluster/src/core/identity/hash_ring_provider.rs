use alloc::vec::Vec;
use core::f64;
use core::hash::{Hash, Hasher};

use libm::log;
use rapidhash::RapidHasher;

use crate::core::config::HashStrategy;

use super::cluster_identity::ClusterIdentity;
use super::cluster_node::ClusterNode;
use super::node_id::NodeId;
use super::topology_snapshot::TopologySnapshot;

/// Provides deterministic node selection via rendezvous hashing.
#[derive(Debug)]
pub struct HashRingProvider {
    strategy: HashStrategy,
    hash_seed: u64,
    nodes: Vec<ClusterNode>,
    last_hash: u64,
}

#[cfg(test)]
mod tests;

impl HashRingProvider {
    /// Creates a provider with the given hashing strategy and seed.
    #[must_use]
    pub fn new(strategy: HashStrategy, hash_seed: u64) -> Self {
        Self { strategy, hash_seed, nodes: Vec::new(), last_hash: 0 }
    }

    /// Updates the internal ring with the provided topology snapshot.
    pub fn rebuild(&mut self, snapshot: &TopologySnapshot) {
        self.nodes.clear();
        self.nodes.extend(
            snapshot
                .members()
                .iter()
                .filter(|node| !node.is_blocked())
                .cloned(),
        );
        self.last_hash = snapshot.hash();
    }

    /// Selects the preferred owner for the given identity.
    #[must_use]
    pub fn select(&self, identity: &ClusterIdentity, requester: &NodeId) -> Option<ClusterNode> {
        if self.nodes.is_empty() {
            return None;
        }

        let mut best: Option<(ClusterNode, f64)> = None;
        for node in &self.nodes {
            let score = self.score_for(node, identity, requester);
            match &mut best {
                Some((_, best_score)) if *best_score >= score => {}
                _ => best = Some((node.clone(), score)),
            }
        }
        best.map(|(node, _)| node)
    }

    /// Returns the hash of the last applied topology snapshot.
    #[must_use]
    pub fn topology_hash(&self) -> u64 {
        self.last_hash
    }

    /// Returns true when the ring contains no members.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Internal helper to access nodes for testing.
    #[must_use]
    pub fn nodes(&self) -> &[ClusterNode] {
        &self.nodes
    }

    fn score_for(&self, node: &ClusterNode, identity: &ClusterIdentity, requester: &NodeId) -> f64 {
        const SCALE: f64 = (u64::MAX as f64) + 1.0;
        let seed = self.hash_seed ^ u64::from(node.weight().max(1));
        let mut hasher = RapidHasher::new(seed);
        identity.hash(&mut hasher);
        requester.hash(&mut hasher);
        node.id().hash(&mut hasher);
        let value = hasher.finish();
        match self.strategy {
            HashStrategy::WeightedRendezvous => {
                let weight = node.weight().max(1) as f64;
                let unit = ((value as f64) + 1.0) / SCALE;
                -log(unit.max(f64::MIN_POSITIVE)) / weight
            }
            HashStrategy::Maglev => value as f64,
            HashStrategy::Rendezvous => value as f64,
        }
    }
}
