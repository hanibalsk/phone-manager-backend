//! Common validation utilities.

use validator::ValidationError;

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
}
