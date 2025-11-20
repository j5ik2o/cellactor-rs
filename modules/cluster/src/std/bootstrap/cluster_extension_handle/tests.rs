use crate::std::bootstrap::{BootstrapState, ClusterExtensionHandle};

#[test]
fn state_is_preserved() {
  let handle = ClusterExtensionHandle::new(BootstrapState::Ready);

  assert_eq!(&BootstrapState::Ready, handle.state());
}
