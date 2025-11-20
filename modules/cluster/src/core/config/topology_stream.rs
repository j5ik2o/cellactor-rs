mod topology_stream_clone;

use topology_stream_clone::TopologyStreamClone;

/// Stream of topology events supplied by a membership provider.
pub trait TopologyStream: TopologyStreamClone {
  /// Describes the stream for debugging purposes.
  fn stream_id(&self) -> &'static str;
}
