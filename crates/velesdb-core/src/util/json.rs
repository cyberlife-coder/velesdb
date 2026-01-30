//! JSON utility functions for working with serde_json::Value.
//!
//! This module provides helper functions to safely extract typed values
//! from JSON payloads.

use serde_json::Value;

/// Extract a timestamp (i64) from a JSON payload.
///
/// Looks for a field named "timestamp" and returns its value if it's an integer.
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use velesdb_core::util::json::timestamp;
///
/// let payload = json!({"timestamp": 1234567890});
/// assert_eq!(timestamp(&payload), Some(1234567890));
/// ```
pub fn timestamp(payload: &Value) -> Option<i64> {
    payload.get("timestamp").and_then(Value::as_i64)
}

/// Extract a string value from a JSON payload by key.
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use velesdb_core::util::json::get_str;
///
/// let payload = json!({"name": "hello"});
/// assert_eq!(get_str(&payload, "name"), Some("hello"));
/// ```
pub fn get_str<'a>(payload: &'a Value, key: &str) -> Option<&'a str> {
    payload.get(key).and_then(Value::as_str)
}

/// Extract an f32 value from a JSON payload by key.
///
/// Converts from f64 (JSON's number type) to f32.
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use velesdb_core::util::json::get_f32;
///
/// let payload = json!({"score": 3.14});
/// let score = get_f32(&payload, "score");
/// assert!(score.is_some());
/// assert!((score.unwrap() - 3.14).abs() < 0.01);
/// ```
pub fn get_f32(payload: &Value, key: &str) -> Option<f32> {
    payload.get(key).and_then(Value::as_f64).map(|v| v as f32)
}
