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
