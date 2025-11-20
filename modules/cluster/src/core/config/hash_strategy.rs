/// Rendezvous hashing strategy used to pick owners.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashStrategy {
    /// Standard rendezvous hashing without weights.
    Rendezvous,
    /// Weighted rendezvous hashing with per-node weights.
    WeightedRendezvous,
    /// Maglev hashing for large node counts.
    Maglev,
}

impl Default for HashStrategy {
    fn default() -> Self {
        Self::Rendezvous
    }
}
