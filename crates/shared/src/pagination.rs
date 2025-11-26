//! Cursor-based pagination utilities.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Utc};
use thiserror::Error;

/// Error type for cursor operations.
#[derive(Debug, Error)]
pub enum CursorError {
    #[error("Invalid cursor format")]
    InvalidFormat,
    #[error("Invalid cursor encoding")]
    InvalidEncoding,
    #[error("Invalid timestamp in cursor")]
    InvalidTimestamp,
    #[error("Invalid ID in cursor")]
    InvalidId,
}

/// Encodes a cursor from timestamp and ID.
///
/// The cursor format is: base64(RFC3339_timestamp:id)
/// This composite cursor handles locations with identical timestamps.
pub fn encode_cursor(captured_at: DateTime<Utc>, id: i64) -> String {
    let raw = format!(
        "{}:{}",
        captured_at.to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
        id
    );
    URL_SAFE_NO_PAD.encode(raw.as_bytes())
}

/// Decodes a cursor into timestamp and ID.
///
/// Returns `(timestamp, id)` tuple on success.
pub fn decode_cursor(cursor: &str) -> Result<(DateTime<Utc>, i64), CursorError> {
    // Decode base64
    let decoded = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| CursorError::InvalidEncoding)?;

    // Convert to string
    let s = String::from_utf8(decoded).map_err(|_| CursorError::InvalidFormat)?;

    // Split on last colon (timestamp may contain colons)
    let colon_pos = s.rfind(':').ok_or(CursorError::InvalidFormat)?;

    let timestamp_str = &s[..colon_pos];
    let id_str = &s[colon_pos + 1..];

    // Parse ID
    let id: i64 = id_str.parse().map_err(|_| CursorError::InvalidId)?;

    // Parse timestamp
    let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
        .map_err(|_| CursorError::InvalidTimestamp)?
        .with_timezone(&Utc);

    Ok((timestamp, id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Timelike};

    #[test]
    fn test_encode_decode_cursor_roundtrip() {
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        let id = 12345i64;

        let cursor = encode_cursor(timestamp, id);
        let (decoded_ts, decoded_id) = decode_cursor(&cursor).unwrap();

        assert_eq!(decoded_ts, timestamp);
        assert_eq!(decoded_id, id);
    }

    #[test]
    fn test_encode_decode_with_microseconds() {
        let timestamp = Utc
            .with_ymd_and_hms(2024, 6, 15, 14, 30, 45)
            .unwrap()
            .with_nanosecond(123456000)
            .unwrap();
        let id = 999999i64;

        let cursor = encode_cursor(timestamp, id);
        let (decoded_ts, decoded_id) = decode_cursor(&cursor).unwrap();

        // Microsecond precision is preserved
        assert_eq!(decoded_ts.timestamp_micros(), timestamp.timestamp_micros());
        assert_eq!(decoded_id, id);
    }

    #[test]
    fn test_decode_invalid_base64() {
        let result = decode_cursor("not-valid-base64!!!");
        assert!(matches!(result, Err(CursorError::InvalidEncoding)));
    }

    #[test]
    fn test_decode_missing_colon() {
        // Valid base64 but no colon separator
        let invalid = URL_SAFE_NO_PAD.encode(b"no-colon-here");
        let result = decode_cursor(&invalid);
        assert!(matches!(result, Err(CursorError::InvalidFormat)));
    }

    #[test]
    fn test_decode_invalid_id() {
        // Valid format but non-numeric ID
        let invalid = URL_SAFE_NO_PAD.encode(b"2024-01-15T10:30:00Z:not-a-number");
        let result = decode_cursor(&invalid);
        assert!(matches!(result, Err(CursorError::InvalidId)));
    }

    #[test]
    fn test_decode_invalid_timestamp() {
        // Valid format but invalid timestamp
        let invalid = URL_SAFE_NO_PAD.encode(b"not-a-timestamp:12345");
        let result = decode_cursor(&invalid);
        assert!(matches!(result, Err(CursorError::InvalidTimestamp)));
    }

    #[test]
    fn test_encode_negative_id() {
        // Edge case: negative ID (shouldn't happen but should handle)
        let timestamp = Utc::now();
        let id = -1i64;

        let cursor = encode_cursor(timestamp, id);
        let (decoded_ts, decoded_id) = decode_cursor(&cursor).unwrap();

        assert_eq!(decoded_ts.timestamp(), timestamp.timestamp());
        assert_eq!(decoded_id, id);
    }

    #[test]
    fn test_encode_large_id() {
        let timestamp = Utc::now();
        let id = i64::MAX;

        let cursor = encode_cursor(timestamp, id);
        let (_, decoded_id) = decode_cursor(&cursor).unwrap();

        assert_eq!(decoded_id, id);
    }

    #[test]
    fn test_cursor_is_url_safe() {
        let timestamp = Utc::now();
        let id = 12345i64;

        let cursor = encode_cursor(timestamp, id);

        // URL_SAFE_NO_PAD should not contain +, /, or =
        assert!(!cursor.contains('+'));
        assert!(!cursor.contains('/'));
        assert!(!cursor.contains('='));
    }
}
