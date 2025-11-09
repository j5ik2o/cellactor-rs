//! Helpers to determine whether a type can be considered a pure value.

/// Returns `true` when the provided type can be treated as a pure value.
#[inline]
pub(super) fn is_pure_value<T>() -> bool {
  !core::mem::needs_drop::<T>()
}
