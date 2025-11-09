//! Pairing of aggregate schema metadata and its field accessors.

use super::{aggregate_accessors::AggregateAccessors, aggregate_schema::AggregateSchema};

/// Registration artefact returned by the schema builder.
#[derive(Clone, Debug)]
pub struct AggregateSchemaRegistration {
  schema:    AggregateSchema,
  accessors: AggregateAccessors,
}

impl AggregateSchemaRegistration {
  /// Creates a new registration bundle.
  #[must_use]
  pub const fn new(schema: AggregateSchema, accessors: AggregateAccessors) -> Self {
    Self { schema, accessors }
  }

  /// Returns an immutable reference to the schema.
  #[must_use]
  pub fn schema(&self) -> &AggregateSchema {
    &self.schema
  }

  /// Returns an immutable reference to the accessors.
  #[must_use]
  pub fn accessors(&self) -> &AggregateAccessors {
    &self.accessors
  }

  /// Deconstructs the registration into its parts.
  #[must_use]
  pub fn into_parts(self) -> (AggregateSchema, AggregateAccessors) {
    (self.schema, self.accessors)
  }
}
