//! Utility module for common helper functions and macros.
//!
//! This module provides:
//! - Safe type conversion macros (`checked_u32`)
//! - JSON helper functions for serde_json::Value
//! - Checksum utilities (CRC32)

pub mod checksum;
pub mod convert;
pub mod json;

pub use checksum::crc32;
pub use convert::checked_u32;
pub use json::{get_f32, get_str, timestamp};
