//! Logical identifier for a provider entry.

use alloc::string::String;

use serde::{Deserialize, Serialize};

/// Logical identifier for a provider entry.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProviderId(String);

impl ProviderId {
  /// Creates a new provider id.
  #[must_use]
  pub fn new(id: impl Into<String>) -> Self {
    Self(id.into())
  }

  /// Returns the underlying id as str.
  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.0
  }
}
