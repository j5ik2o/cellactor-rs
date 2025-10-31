use alloc::{borrow::Cow, string::String};
use core::fmt;

/// Human readable explanation associated with an actor error.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ActorErrorReason(Cow<'static, str>);

impl ActorErrorReason {
  /// Creates a new reason.
  #[must_use]
  pub fn new(reason: impl Into<Cow<'static, str>>) -> Self {
    Self(reason.into())
  }

  /// Returns the message.
  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl From<&'static str> for ActorErrorReason {
  fn from(value: &'static str) -> Self {
    Self(Cow::Borrowed(value))
  }
}

impl From<String> for ActorErrorReason {
  fn from(value: String) -> Self {
    Self(Cow::Owned(value))
  }
}

impl From<Cow<'static, str>> for ActorErrorReason {
  fn from(value: Cow<'static, str>) -> Self {
    Self(value)
  }
}

impl fmt::Display for ActorErrorReason {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(&self.0)
  }
}
