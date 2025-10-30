use alloc::{borrow::ToOwned, format, string::String};

use hashbrown::HashMap;

use super::name_registry_error::NameRegistryError;
use crate::pid::Pid;

/// Tracks actor names within a parent scope.
#[derive(Default)]
pub struct NameRegistry {
  entries: HashMap<String, Pid>,
}

impl NameRegistry {
  /// Creates a new, empty registry.
  #[must_use]
  pub fn new() -> Self {
    Self { entries: HashMap::new() }
  }

  /// Attempts to register a name for the provided pid.
  ///
  /// # Errors
  ///
  /// Returns an error if the name is already registered.
  pub fn register(&mut self, name: &str, pid: Pid) -> Result<(), NameRegistryError> {
    if self.entries.contains_key(name) {
      return Err(NameRegistryError::Duplicate(name.to_owned()));
    }
    self.entries.insert(name.to_owned(), pid);
    Ok(())
  }

  /// Resolves a name to its associated pid.
  #[must_use]
  pub fn resolve(&self, name: &str) -> Option<Pid> {
    self.entries.get(name).copied()
  }

  /// Removes the name from the registry.
  pub fn remove(&mut self, name: &str) -> Option<Pid> {
    self.entries.remove(name)
  }

  /// Generates an anonymous name derived from the pid.
  #[must_use]
  pub fn generate_anonymous(&self, pid: Pid) -> String {
    format!("anon-{}", pid)
  }
}
