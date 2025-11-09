//! Serialization infrastructure built on top of extensions.

/// Aggregate schema representation.
mod aggregate_schema;
/// Aggregate schema builder API.
mod aggregate_schema_builder;
/// Holds field extractors tied to aggregate schemas.
mod aggregate_accessors;
/// Serializer implementations including the built-in bincode backend.
mod bincode_serializer;
/// Thin wrapper over owned byte buffers.
mod bytes;
/// Shared serialization constants.
mod constants;
/// Envelope handling description.
mod envelope_mode;
/// Error types for serialization failures.
mod error;
/// Extension entry point and ActorSystem integration.
mod extension;
/// External serializer policy cache.
mod external_serializer_policy;
/// Field metadata types.
mod field_node;
mod field_options;
mod field_payload;
mod field_path;
mod field_path_display;
mod field_path_hash;
mod field_path_segment;
mod field_value_ref;
mod field_envelope_builder;
mod field_traversal_engine;
/// Serializable payload container storing manifest metadata.
mod payload;
/// Marker trait for Pekko default serialization.
mod pekko_serializable;
/// Nested serializer orchestrator implementation.
mod nested_serializer_orchestrator;
/// Helpers for pure-value checks.
mod pure_value;
/// Serializer and manifest registries.
mod registry;
/// Registry audit reporting utilities.
mod registry_audit;
/// Telemetry hooks for nested serialization.
mod serialization_telemetry;
/// Object-safe serializer traits and handles.
mod serializer;
/// Traversal policy definitions.
mod traversal_policy;
/// Type binding for serialization.
mod type_binding;
/// Aggregate schema registration bundle.
mod aggregate_schema_registration;
/// Field accessor trait objects for aggregates.
mod aggregate_field_extractor;

pub use aggregate_accessors::AggregateAccessors;
pub use aggregate_schema::AggregateSchema;
pub use aggregate_schema_builder::AggregateSchemaBuilder;
pub use aggregate_schema_registration::AggregateSchemaRegistration;
pub use bincode_serializer::BincodeSerializer;
pub use bytes::Bytes;
pub use envelope_mode::EnvelopeMode;
pub use error::SerializationError;
pub use extension::{SERIALIZATION_EXTENSION, Serialization, SerializationExtensionId};
pub use field_node::FieldNode;
pub use field_options::FieldOptions;
pub use field_payload::FieldPayload;
pub use field_path::FieldPath;
pub use field_path_display::FieldPathDisplay;
pub use field_path_hash::FieldPathHash;
pub use field_path_segment::FieldPathSegment;
pub use field_value_ref::FieldValueRef;
pub use field_envelope_builder::FieldEnvelopeBuilder;
pub use field_traversal_engine::{FieldTraversalEngine, FieldTraversalPlan};
pub use payload::SerializedPayload;
pub use pekko_serializable::PekkoSerializable;
pub use registry::{PekkoAssignmentMetrics, SerializerRegistry};
pub use registry_audit::{RegistryAuditIssue, RegistryAuditReport};
pub use serialization_telemetry::{NoopSerializationTelemetry, SerializationTelemetry};
pub use serializer::{SerializerHandle, SerializerImpl};
pub use traversal_policy::TraversalPolicy;
