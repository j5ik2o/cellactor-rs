//! Hash utilities for field paths.

use alloc::vec::Vec;

use super::{constants::hash_field_bytes, field_path::FieldPath, field_path_display::FieldPathDisplay};

/// Hash value derived from both numeric path segments and UTF-8 display string.
pub type FieldPathHash = u128;

/// Computes the hash for the provided path metadata.
#[must_use]
pub(super) fn compute_field_path_hash(path: &FieldPath, display: &FieldPathDisplay) -> FieldPathHash {
  let mut buffer = Vec::with_capacity(path.segments().len() * core::mem::size_of::<u16>());
  for segment in path.segments() {
    buffer.extend_from_slice(&segment.value().to_le_bytes());
  }
  let mut hash = hash_field_bytes(buffer.as_slice());
  hash ^= hash_field_bytes(display.as_bytes());
  hash
}
