//! Safe type conversion macros.
//!
//! This module provides macros for safe numeric conversions with bounds checking.

/// Macro to safely convert a u64 to u32 with bounds checking.
///
/// Panics if the value exceeds `u32::MAX`.
///
/// # Examples
///
/// ```
/// use velesdb_core::checked_u32;
///
/// let value: u64 = 100;
/// let result = checked_u32!(value, "document ID");
/// assert_eq!(result, 100u32);
/// ```
#[macro_export]
macro_rules! checked_u32 {
    ($value:expr, $context:expr) => {{
        let v: u64 = $value;
        #[allow(clippy::checked_conversions)]
        {
            assert!(v <= u32::MAX as u64, "{} {} exceeds u32::MAX", $context, v);
            v as u32
        }
    }};
}

pub use checked_u32;
