//! Common validation utilities.

use chrono::{TimeZone, Utc};
use validator::ValidationError;

/// Maximum age of a timestamp in days (7 days).
const MAX_TIMESTAMP_AGE_DAYS: i64 = 7;

/// Maximum allowed future timestamp tolerance in seconds (5 minutes for clock skew).
const MAX_FUTURE_TOLERANCE_SECS: i64 = 300;

/// Validates that a latitude value is within valid range (-90 to 90).
pub fn validate_latitude(lat: f64) -> Result<(), ValidationError> {
    if (-90.0..=90.0).contains(&lat) {
        Ok(())
    } else {
        let mut err = ValidationError::new("latitude_range");
        err.message = Some("Latitude must be between -90 and 90".into());
        Err(err)
    }
}

/// Validates that a longitude value is within valid range (-180 to 180).
pub fn validate_longitude(lon: f64) -> Result<(), ValidationError> {
    if (-180.0..=180.0).contains(&lon) {
        Ok(())
    } else {
        let mut err = ValidationError::new("longitude_range");
        err.message = Some("Longitude must be between -180 and 180".into());
        Err(err)
    }
}

/// Validates that accuracy is non-negative.
pub fn validate_accuracy(accuracy: f64) -> Result<(), ValidationError> {
    if accuracy >= 0.0 {
        Ok(())
    } else {
        let mut err = ValidationError::new("accuracy_range");
        err.message = Some("Accuracy must be non-negative".into());
        Err(err)
    }
}

/// Validates that bearing is within valid range (0 to 360).
pub fn validate_bearing(bearing: f64) -> Result<(), ValidationError> {
    if (0.0..=360.0).contains(&bearing) {
        Ok(())
    } else {
        let mut err = ValidationError::new("bearing_range");
        err.message = Some("Bearing must be between 0 and 360".into());
        Err(err)
    }
}

/// Validates that speed is non-negative.
pub fn validate_speed(speed: f64) -> Result<(), ValidationError> {
    if speed >= 0.0 {
        Ok(())
    } else {
        let mut err = ValidationError::new("speed_range");
        err.message = Some("Speed must be non-negative".into());
        Err(err)
    }
}

/// Validates that battery level is within valid range (0 to 100).
pub fn validate_battery_level(level: i32) -> Result<(), ValidationError> {
    if (0..=100).contains(&level) {
        Ok(())
    } else {
        let mut err = ValidationError::new("battery_range");
        err.message = Some("Battery level must be between 0 and 100".into());
        Err(err)
    }
}

/// Validates that a timestamp (in milliseconds since epoch) is within acceptable range.
/// - Must not be more than 5 minutes in the future (allows for clock skew)
/// - Must not be older than 7 days
pub fn validate_timestamp(timestamp_millis: i64) -> Result<(), ValidationError> {
    let now = Utc::now();

    // Convert millisecond timestamp to DateTime
    let timestamp = match Utc.timestamp_millis_opt(timestamp_millis).single() {
        Some(ts) => ts,
        None => {
            let mut err = ValidationError::new("timestamp_invalid");
            err.message = Some("Invalid timestamp format".into());
            return Err(err);
        }
    };

    // Check if timestamp is too far in the future
    let future_limit = now + chrono::Duration::seconds(MAX_FUTURE_TOLERANCE_SECS);
    if timestamp > future_limit {
        let mut err = ValidationError::new("timestamp_future");
        err.message = Some("Timestamp cannot be in the future".into());
        return Err(err);
    }

    // Check if timestamp is too old
    let past_limit = now - chrono::Duration::days(MAX_TIMESTAMP_AGE_DAYS);
    if timestamp < past_limit {
        let mut err = ValidationError::new("timestamp_old");
        err.message = Some("Timestamp cannot be older than 7 days".into());
        return Err(err);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Latitude tests
    #[test]
    fn test_validate_latitude() {
        assert!(validate_latitude(0.0).is_ok());
        assert!(validate_latitude(90.0).is_ok());
        assert!(validate_latitude(-90.0).is_ok());
        assert!(validate_latitude(90.1).is_err());
        assert!(validate_latitude(-90.1).is_err());
    }

    #[test]
    fn test_validate_latitude_decimals() {
        assert!(validate_latitude(45.123456).is_ok());
        assert!(validate_latitude(-45.123456).is_ok());
        assert!(validate_latitude(89.999999).is_ok());
    }

    #[test]
    fn test_validate_latitude_error_message() {
        let err = validate_latitude(100.0).unwrap_err();
        assert_eq!(
            err.message.unwrap().to_string(),
            "Latitude must be between -90 and 90"
        );
    }

    // Longitude tests
    #[test]
    fn test_validate_longitude() {
        assert!(validate_longitude(0.0).is_ok());
        assert!(validate_longitude(180.0).is_ok());
        assert!(validate_longitude(-180.0).is_ok());
        assert!(validate_longitude(180.1).is_err());
        assert!(validate_longitude(-180.1).is_err());
    }

    #[test]
    fn test_validate_longitude_decimals() {
        assert!(validate_longitude(122.123456).is_ok());
        assert!(validate_longitude(-122.123456).is_ok());
        assert!(validate_longitude(179.999999).is_ok());
    }

    #[test]
    fn test_validate_longitude_error_message() {
        let err = validate_longitude(200.0).unwrap_err();
        assert_eq!(
            err.message.unwrap().to_string(),
            "Longitude must be between -180 and 180"
        );
    }

    // Accuracy tests
    #[test]
    fn test_validate_accuracy() {
        assert!(validate_accuracy(0.0).is_ok());
        assert!(validate_accuracy(100.0).is_ok());
        assert!(validate_accuracy(-1.0).is_err());
    }

    #[test]
    fn test_validate_accuracy_large_values() {
        assert!(validate_accuracy(10000.0).is_ok());
        assert!(validate_accuracy(0.001).is_ok());
    }

    #[test]
    fn test_validate_accuracy_error_message() {
        let err = validate_accuracy(-5.0).unwrap_err();
        assert_eq!(
            err.message.unwrap().to_string(),
            "Accuracy must be non-negative"
        );
    }

    // Bearing tests
    #[test]
    fn test_validate_bearing() {
        assert!(validate_bearing(0.0).is_ok());
        assert!(validate_bearing(360.0).is_ok());
        assert!(validate_bearing(180.0).is_ok());
        assert!(validate_bearing(-1.0).is_err());
        assert!(validate_bearing(360.1).is_err());
    }

    #[test]
    fn test_validate_bearing_common_directions() {
        assert!(validate_bearing(0.0).is_ok()); // North
        assert!(validate_bearing(90.0).is_ok()); // East
        assert!(validate_bearing(180.0).is_ok()); // South
        assert!(validate_bearing(270.0).is_ok()); // West
    }

    #[test]
    fn test_validate_bearing_error_message() {
        let err = validate_bearing(400.0).unwrap_err();
        assert_eq!(
            err.message.unwrap().to_string(),
            "Bearing must be between 0 and 360"
        );
    }

    // Speed tests
    #[test]
    fn test_validate_speed() {
        assert!(validate_speed(0.0).is_ok());
        assert!(validate_speed(100.0).is_ok());
        assert!(validate_speed(-1.0).is_err());
    }

    #[test]
    fn test_validate_speed_realistic_values() {
        assert!(validate_speed(5.5).is_ok()); // Walking
        assert!(validate_speed(27.8).is_ok()); // 100 km/h
        assert!(validate_speed(0.001).is_ok()); // Very slow
    }

    #[test]
    fn test_validate_speed_error_message() {
        let err = validate_speed(-10.0).unwrap_err();
        assert_eq!(
            err.message.unwrap().to_string(),
            "Speed must be non-negative"
        );
    }

    // Battery level tests
    #[test]
    fn test_validate_battery_level() {
        assert!(validate_battery_level(0).is_ok());
        assert!(validate_battery_level(100).is_ok());
        assert!(validate_battery_level(50).is_ok());
        assert!(validate_battery_level(-1).is_err());
        assert!(validate_battery_level(101).is_err());
    }

    #[test]
    fn test_validate_battery_level_edge_cases() {
        assert!(validate_battery_level(1).is_ok());
        assert!(validate_battery_level(99).is_ok());
        assert!(validate_battery_level(-100).is_err());
        assert!(validate_battery_level(200).is_err());
    }

    #[test]
    fn test_validate_battery_level_error_message() {
        let err = validate_battery_level(150).unwrap_err();
        assert_eq!(
            err.message.unwrap().to_string(),
            "Battery level must be between 0 and 100"
        );
    }

    // Timestamp tests
    #[test]
    fn test_validate_timestamp_current() {
        let now_millis = Utc::now().timestamp_millis();
        assert!(validate_timestamp(now_millis).is_ok());
    }

    #[test]
    fn test_validate_timestamp_recent_past() {
        // 1 hour ago should be valid
        let one_hour_ago = Utc::now() - chrono::Duration::hours(1);
        assert!(validate_timestamp(one_hour_ago.timestamp_millis()).is_ok());

        // 1 day ago should be valid
        let one_day_ago = Utc::now() - chrono::Duration::days(1);
        assert!(validate_timestamp(one_day_ago.timestamp_millis()).is_ok());

        // 6 days ago should be valid
        let six_days_ago = Utc::now() - chrono::Duration::days(6);
        assert!(validate_timestamp(six_days_ago.timestamp_millis()).is_ok());
    }

    #[test]
    fn test_validate_timestamp_too_old() {
        // 8 days ago should be invalid
        let eight_days_ago = Utc::now() - chrono::Duration::days(8);
        assert!(validate_timestamp(eight_days_ago.timestamp_millis()).is_err());

        // 30 days ago should be invalid
        let thirty_days_ago = Utc::now() - chrono::Duration::days(30);
        assert!(validate_timestamp(thirty_days_ago.timestamp_millis()).is_err());
    }

    #[test]
    fn test_validate_timestamp_slight_future() {
        // 1 minute in the future should be valid (clock skew tolerance)
        let one_min_future = Utc::now() + chrono::Duration::minutes(1);
        assert!(validate_timestamp(one_min_future.timestamp_millis()).is_ok());

        // 4 minutes in the future should be valid
        let four_min_future = Utc::now() + chrono::Duration::minutes(4);
        assert!(validate_timestamp(four_min_future.timestamp_millis()).is_ok());
    }

    #[test]
    fn test_validate_timestamp_too_far_future() {
        // 10 minutes in the future should be invalid
        let ten_min_future = Utc::now() + chrono::Duration::minutes(10);
        assert!(validate_timestamp(ten_min_future.timestamp_millis()).is_err());

        // 1 hour in the future should be invalid
        let one_hour_future = Utc::now() + chrono::Duration::hours(1);
        assert!(validate_timestamp(one_hour_future.timestamp_millis()).is_err());
    }

    #[test]
    fn test_validate_timestamp_future_error_message() {
        let far_future = Utc::now() + chrono::Duration::hours(1);
        let err = validate_timestamp(far_future.timestamp_millis()).unwrap_err();
        assert_eq!(
            err.message.unwrap().to_string(),
            "Timestamp cannot be in the future"
        );
    }

    #[test]
    fn test_validate_timestamp_old_error_message() {
        let old_timestamp = Utc::now() - chrono::Duration::days(10);
        let err = validate_timestamp(old_timestamp.timestamp_millis()).unwrap_err();
        assert_eq!(
            err.message.unwrap().to_string(),
            "Timestamp cannot be older than 7 days"
        );
    }

    #[test]
    fn test_validate_timestamp_boundary_7_days() {
        // Just under 7 days should be valid
        let just_under_7_days = Utc::now() - chrono::Duration::days(7) + chrono::Duration::hours(1);
        assert!(validate_timestamp(just_under_7_days.timestamp_millis()).is_ok());
    }
}
