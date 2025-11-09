//! Events emitted by serialization subsystems.

use alloc::vec::Vec;

use super::serialization_audit_issue::SerializationAuditIssue;
use crate::serialization::RegistryAuditReport;

/// Event summarising the result of a serialization schema audit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SerializationAuditEvent {
  /// Indicates whether the audit completed without issues.
  pub success:         bool,
  /// Number of schemas that were inspected.
  pub schemas_checked: usize,
  /// Detailed issue list (empty when `success` is true).
  pub issues:          Vec<SerializationAuditIssue>,
}

impl SerializationAuditEvent {
  /// Returns true when no schema issues were detected.
  #[must_use]
  pub fn success(&self) -> bool {
    self.success
  }
}

impl From<&RegistryAuditReport> for SerializationAuditEvent {
  fn from(report: &RegistryAuditReport) -> Self {
    Self {
      success:         report.success(),
      schemas_checked: report.schemas_checked,
      issues:          report
        .issues
        .iter()
        .map(|issue| SerializationAuditIssue {
          field_path: issue.field_path.clone(),
          type_name:  issue.type_name,
          reason:     issue.reason.clone(),
        })
        .collect(),
    }
  }
}
