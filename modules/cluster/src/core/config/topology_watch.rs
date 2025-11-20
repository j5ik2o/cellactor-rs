use alloc::boxed::Box;
use core::fmt;

use super::topology_stream::TopologyStream;

/// Handle to a topology update stream.
pub struct TopologyWatch {
  stream: Box<dyn TopologyStream>,
}

impl TopologyWatch {
  /// Wraps a topology stream inside a watch handle.
  pub fn new(stream: Box<dyn TopologyStream>) -> Self {
    Self { stream }
  }

  /// Returns the identifier of the underlying stream for diagnostics.
  pub fn stream_id(&self) -> &'static str {
    self.stream.stream_id()
  }

  /// Provides immutable access to the underlying stream trait object.
  pub fn stream(&self) -> &dyn TopologyStream {
    self.stream.as_ref()
  }
}

impl Clone for TopologyWatch {
  fn clone(&self) -> Self {
    Self { stream: self.stream.clone() }
  }
}

impl fmt::Debug for TopologyWatch {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("TopologyWatch").field("stream_id", &self.stream.stream_id()).finish()
  }
}
