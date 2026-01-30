//! Tests for the util module.

use serde_json::json;

#[test]
fn test_crc32_hello() {
    use crate::util::checksum::crc32;
    assert_eq!(crc32(b"hello"), 0x3610_a686);
}

#[test]
fn test_crc32_empty() {
    use crate::util::checksum::crc32;
    assert_eq!(crc32(b""), 0x0000_0000);
}

#[test]
fn test_crc32_single_byte() {
    use crate::util::checksum::crc32;
    assert_eq!(crc32(b"a"), 0xe8b7_be43);
}

#[test]
fn test_crc32_longer_string() {
    use crate::util::checksum::crc32;
    assert_eq!(
        crc32(b"The quick brown fox jumps over the lazy dog"),
        0x414f_a339
    );
}

#[test]
fn test_crc32_binary_data() {
    use crate::util::checksum::crc32;
    let data: Vec<u8> = (0..=255).collect();
    assert_eq!(crc32(&data), 0x2905_8c73);
}

#[test]
fn test_checked_u32_valid() {
    use crate::checked_u32;
    assert_eq!(checked_u32!(100u64, "test"), 100u32);
    assert_eq!(checked_u32!(0u64, "test"), 0u32);
    assert_eq!(checked_u32!(u32::MAX as u64, "test"), u32::MAX);
}

#[test]
#[should_panic(expected = "exceeds u32::MAX")]
fn test_checked_u32_overflow() {
    use crate::checked_u32;
    checked_u32!((u32::MAX as u64) + 1, "test");
}

#[test]
fn test_timestamp_valid() {
    use crate::util::json::timestamp;
    assert_eq!(
        timestamp(&json!({"timestamp": 1_234_567_890})),
        Some(1_234_567_890)
    );
    assert_eq!(timestamp(&json!({"timestamp": 0})), Some(0));
    assert_eq!(timestamp(&json!({"timestamp": -1})), Some(-1));
}

#[test]
fn test_timestamp_missing() {
    use crate::util::json::timestamp;
    assert_eq!(timestamp(&json!({})), None);
    assert_eq!(timestamp(&json!({"other": 123})), None);
}

#[test]
fn test_timestamp_not_integer() {
    use crate::util::json::timestamp;
    assert_eq!(timestamp(&json!({"timestamp": "not_a_number"})), None);
    assert_eq!(timestamp(&json!({"timestamp": 2.5})), None);
    assert_eq!(timestamp(&json!({"timestamp": null})), None);
}

#[test]
fn test_get_str_valid() {
    use crate::util::json::get_str;
    assert_eq!(get_str(&json!({"name": "hello"}), "name"), Some("hello"));
    assert_eq!(get_str(&json!({"name": ""}), "name"), Some(""));
}

#[test]
fn test_get_str_missing() {
    use crate::util::json::get_str;
    assert_eq!(get_str(&json!({}), "name"), None);
    assert_eq!(get_str(&json!({"other": "value"}), "name"), None);
}

#[test]
fn test_get_str_not_string() {
    use crate::util::json::get_str;
    assert_eq!(get_str(&json!({"name": 123}), "name"), None);
    assert_eq!(get_str(&json!({"name": null}), "name"), None);
}

#[test]
fn test_get_f32_valid() {
    use crate::util::json::get_f32;
    let result = get_f32(&json!({"score": 2.5}), "score");
    assert!(result.is_some());
    assert!((result.unwrap() - 2.5f32).abs() < 0.01);
}

#[test]
fn test_get_f32_integer() {
    use crate::util::json::get_f32;
    let result = get_f32(&json!({"score": 42}), "score");
    assert!(result.is_some());
    assert!((result.unwrap() - 42.0f32).abs() < 0.01);
}

#[test]
fn test_get_f32_missing() {
    use crate::util::json::get_f32;
    assert_eq!(get_f32(&json!({}), "score"), None);
}

#[test]
fn test_get_f32_not_number() {
    use crate::util::json::get_f32;
    assert_eq!(get_f32(&json!({"score": "not_a_number"}), "score"), None);
    assert_eq!(get_f32(&json!({"score": null}), "score"), None);
}
