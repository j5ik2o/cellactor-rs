/// Rendezvous hashing strategy used to pick owners.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HashStrategy {
  /// Standard rendezvous hashing without weights.
  #[default]
  Rendezvous,
  /// Weighted rendezvous hashing with per-node weights.
  WeightedRendezvous,
  /// Maglev hashing for large node counts.
  Maglev,
}
