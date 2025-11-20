use fraktor_utils_rs::core::runtime_toolbox::NoStdToolbox;

use crate::core::activation::{ActivationLedger, LedgerError, LeaseStatus};
use crate::core::identity::{ClusterIdentity, NodeId};

#[test]
fn acquires_and_releases_lease() {
    let ledger = ActivationLedger::<NoStdToolbox>::new();
    let identity = ClusterIdentity::new("echo", "a");
    let owner = NodeId::new("node-a");

    let lease = ledger
        .acquire(identity.clone(), owner.clone(), 13)
        .expect("lease");

    assert_eq!(lease.status(), LeaseStatus::Active);
    assert_eq!(lease.owner(), &owner);
    assert!(ledger.release(&identity, lease.lease_id()));
    assert!(ledger.get(&identity).is_none());
}

#[test]
fn rejects_duplicate_acquire() {
    let ledger = ActivationLedger::<NoStdToolbox>::new();
    let identity = ClusterIdentity::new("echo", "a");

    let _ = ledger
        .acquire(identity.clone(), NodeId::new("node-a"), 7)
        .expect("first lease");

    let err = ledger
        .acquire(identity.clone(), NodeId::new("node-b"), 7)
        .expect_err("duplicate should fail");

    match err {
        LedgerError::AlreadyOwned { existing } => {
            assert_eq!(existing.owner().as_str(), "node-a");
        }
    }
}

#[test]
fn revoke_by_owner_removes_matching_leases() {
    let ledger = ActivationLedger::<NoStdToolbox>::new();
    let identity_a = ClusterIdentity::new("echo", "a");
    let identity_b = ClusterIdentity::new("echo", "b");

    let lease_a = ledger
        .acquire(identity_a.clone(), NodeId::new("node-a"), 5)
        .expect("lease a");
    let _ = ledger
        .acquire(identity_b.clone(), NodeId::new("node-b"), 5)
        .expect("lease b");

    let revoked = ledger.revoke_owner(&NodeId::new("node-a"));

    assert_eq!(revoked.len(), 1);
    assert_eq!(revoked[0].0, identity_a);
    assert_eq!(revoked[0].1.lease_id(), lease_a.lease_id());
    assert!(matches!(revoked[0].1.status(), LeaseStatus::Revoked));
    assert!(ledger.get(&identity_a).is_none());
    assert!(ledger.get(&identity_b).is_some());
}
