//! Telemetry helpers for serialization audit reporting.

mod noop_serialization_audit_notifier;
mod serialization_audit_notifier;

pub use noop_serialization_audit_notifier::NoopSerializationAuditNotifier;
pub use serialization_audit_notifier::SerializationAuditNotifier;
