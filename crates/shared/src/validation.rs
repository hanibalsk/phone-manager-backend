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

    #[test]
    fn test_validate_latitude() {
        assert!(validate_latitude(0.0).is_ok());
        assert!(validate_latitude(90.0).is_ok());
        assert!(validate_latitude(-90.0).is_ok());
        assert!(validate_latitude(90.1).is_err());
        assert!(validate_latitude(-90.1).is_err());
    }

    #[test]
    fn test_validate_longitude() {
        assert!(validate_longitude(0.0).is_ok());
        assert!(validate_longitude(180.0).is_ok());
        assert!(validate_longitude(-180.0).is_ok());
        assert!(validate_longitude(180.1).is_err());
        assert!(validate_longitude(-180.1).is_err());
    }

    #[test]
    fn test_validate_accuracy() {
        assert!(validate_accuracy(0.0).is_ok());
        assert!(validate_accuracy(100.0).is_ok());
        assert!(validate_accuracy(-1.0).is_err());
    }

    #[test]
    fn test_validate_battery_level() {
        assert!(validate_battery_level(0).is_ok());
        assert!(validate_battery_level(100).is_ok());
        assert!(validate_battery_level(50).is_ok());
        assert!(validate_battery_level(-1).is_err());
        assert!(validate_battery_level(101).is_err());
    }
}
