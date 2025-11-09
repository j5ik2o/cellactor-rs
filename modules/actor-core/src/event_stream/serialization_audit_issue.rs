//! Issue captured during serialization registry audits.

use alloc::string::String;

/// Detailed record for a schema-related problem detected by the audit flow.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SerializationAuditIssue {
  /// Dot-separated field path identifying the problematic field.
  pub field_path: String,
  /// Name of the field type missing a binding.
  pub type_name:  &'static str,
  /// Short description of the issue.
  pub reason:     String,
}
