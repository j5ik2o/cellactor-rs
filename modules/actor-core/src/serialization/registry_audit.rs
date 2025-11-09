//! Registry audit report structures.

use alloc::{string::String, vec::Vec};

/// Details about a schema validation issue.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryAuditIssue {
  /// Dot-separated field path identifying the problematic entry.
  pub field_path: String,
  /// Name of the field type missing a binding.
  pub type_name:  &'static str,
  /// Short description of the failure reason.
  pub reason:     String,
}

/// Summary of schema validation results.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryAuditReport {
  /// Number of schemas that were inspected.
  pub schemas_checked: usize,
  /// List of detected issues.
  pub issues:          Vec<RegistryAuditIssue>,
}

impl RegistryAuditReport {
  /// Creates a new report from the provided data.
  pub fn new(schemas_checked: usize, issues: Vec<RegistryAuditIssue>) -> Self {
    Self { schemas_checked, issues }
  }

  /// Returns true when no issues were detected.
  #[must_use]
  pub fn success(&self) -> bool {
    self.issues.is_empty()
  }
}
