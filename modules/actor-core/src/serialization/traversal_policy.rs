//! Specifies how aggregate fields should be traversed.

/// Traversal order for nested serialization.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TraversalPolicy {
  /// Depth-first traversal (default).
  DepthFirst,
  /// Breadth-first traversal.
  BreadthFirst,
}
