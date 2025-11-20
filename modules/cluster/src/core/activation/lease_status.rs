/// Represents the lifecycle state of an activation lease.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeaseStatus {
    /// Lease is actively held by an owner.
    Active,
    /// Lease is being released gracefully.
    Releasing,
    /// Lease has been fully released.
    Released,
    /// Lease has been revoked due to topology or block list updates.
    Revoked,
}
