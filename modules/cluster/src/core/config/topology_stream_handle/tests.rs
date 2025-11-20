use crate::core::config::TopologyStreamHandle;

#[test]
fn handle_marks_and_reports_closed() {
  let handle = TopologyStreamHandle::new();

  assert!(!handle.is_closed());
  let prev = handle.mark_closed();
  assert!(!prev);
  assert!(handle.is_closed());
}
