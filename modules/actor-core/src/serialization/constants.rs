//! Serialization-specific limits and hashing helpers.

/// Maximum depth for a field path (number of segments).
pub(super) const MAX_FIELD_PATH_DEPTH: usize = 32;
/// Maximum number of fields supported per aggregate schema.
pub(super) const MAX_FIELDS_PER_AGGREGATE: usize = 128;
/// Maximum UTF-8 bytes allowed for a field path display string.
pub(super) const MAX_FIELD_PATH_BYTES: usize = 96;

const FIELD_PATH_HASH_OFFSET: u128 = 0xcbf2_9ce4_8422_2325_0000_0000_0000_0000;
const FIELD_PATH_HASH_PRIME: u128 = 0x1000_0000_0000_0000_0000_001b_3;

/// Computes a simple FNV-1a style hash for field path metadata.
#[inline]
pub(super) fn hash_field_bytes(bytes: &[u8]) -> u128 {
  let mut hash = FIELD_PATH_HASH_OFFSET;
  for byte in bytes {
    hash ^= u128::from(*byte);
    hash = hash.wrapping_mul(FIELD_PATH_HASH_PRIME);
  }
  hash
}
