/// Reasons explaining why the orchestrator entered a fallback path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SerializationFallbackReason {
  /// No serializer binding was registered for the field type.
  MissingSerializer,
  /// External serializers are not allowed for the field path.
  ExternalNotAllowed,
}
