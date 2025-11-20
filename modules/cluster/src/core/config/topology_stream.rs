use alloc::boxed::Box;

/// Stream of topology events supplied by a membership provider.
pub trait TopologyStream: TopologyStreamClone {
  /// Describes the stream for debugging purposes.
  fn stream_id(&self) -> &'static str;
}

/// Helper trait to make boxed clone available for trait objects.
pub trait TopologyStreamClone {
  /// Clones the stream into a boxed trait object.
  fn clone_box(&self) -> Box<dyn TopologyStream>;
}

impl<T> TopologyStreamClone for T
where
  T: 'static + TopologyStream + Clone,
{
  fn clone_box(&self) -> Box<dyn TopologyStream> {
    Box::new(self.clone())
  }
}

impl Clone for Box<dyn TopologyStream> {
  fn clone(&self) -> Self {
    self.clone_box()
  }
}
