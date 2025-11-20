/// Opaque identifier assigned to each activation lease.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LeaseId(u64);

impl LeaseId {
  /// Creates a new lease identifier.
  #[must_use]
  pub const fn new(value: u64) -> Self {
    Self(value)
  }

  /// Returns the numeric representation.
  #[must_use]
  pub const fn get(self) -> u64 {
    self.0
  }
}
