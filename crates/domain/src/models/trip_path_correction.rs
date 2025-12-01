//! Trip path correction domain model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

// ============================================================================
// Correction Status Enum
// ============================================================================

/// Status of a path correction operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CorrectionStatus {
    /// Correction is pending (not yet processed)
    Pending,
    /// Correction completed successfully
    Completed,
    /// Correction failed (service error)
    Failed,
    /// Correction skipped (e.g., too few points, service unavailable)
    Skipped,
}

impl CorrectionStatus {
    /// Returns the string representation for database storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            CorrectionStatus::Pending => "PENDING",
            CorrectionStatus::Completed => "COMPLETED",
            CorrectionStatus::Failed => "FAILED",
            CorrectionStatus::Skipped => "SKIPPED",
        }
    }
}

impl fmt::Display for CorrectionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for CorrectionStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PENDING" => Ok(CorrectionStatus::Pending),
            "COMPLETED" => Ok(CorrectionStatus::Completed),
            "FAILED" => Ok(CorrectionStatus::Failed),
            "SKIPPED" => Ok(CorrectionStatus::Skipped),
            _ => Err(format!(
                "Invalid correction status: {}. Must be one of: PENDING, COMPLETED, FAILED, SKIPPED",
                s
            )),
        }
    }
}

// ============================================================================
// Core Model
// ============================================================================

/// Represents a trip path correction record.
///
/// Stores the original GPS trace and optionally the map-matched corrected path.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TripPathCorrection {
    pub id: Uuid,
    pub trip_id: Uuid,
    /// Original GPS path as GeoJSON LineString
    pub original_path: String,
    /// Map-matched path as GeoJSON LineString (null if not yet corrected)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corrected_path: Option<String>,
    /// Confidence metric from map-matching service (0.0-1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correction_quality: Option<f32>,
    /// Status of the correction operation
    pub correction_status: CorrectionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Response DTOs
// ============================================================================

/// Response for GET /api/v1/trips/:tripId/path
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct TripPathResponse {
    /// Original path as array of [lat, lon] pairs
    pub original_path: Vec<[f64; 2]>,
    /// Corrected path as array of [lat, lon] pairs (null if not corrected)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corrected_path: Option<Vec<[f64; 2]>>,
    /// Status of the correction operation
    pub correction_status: CorrectionStatus,
    /// Confidence metric from map-matching service (0.0-1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correction_quality: Option<f32>,
}

/// Response for POST /api/v1/trips/:tripId/correct-path
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CorrectPathResponse {
    pub status: String,
    pub message: String,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // CorrectionStatus Tests
    // =========================================================================

    #[test]
    fn test_correction_status_as_str() {
        assert_eq!(CorrectionStatus::Pending.as_str(), "PENDING");
        assert_eq!(CorrectionStatus::Completed.as_str(), "COMPLETED");
        assert_eq!(CorrectionStatus::Failed.as_str(), "FAILED");
        assert_eq!(CorrectionStatus::Skipped.as_str(), "SKIPPED");
    }

    #[test]
    fn test_correction_status_from_str() {
        assert_eq!(
            "PENDING".parse::<CorrectionStatus>().unwrap(),
            CorrectionStatus::Pending
        );
        assert_eq!(
            "COMPLETED".parse::<CorrectionStatus>().unwrap(),
            CorrectionStatus::Completed
        );
        assert_eq!(
            "FAILED".parse::<CorrectionStatus>().unwrap(),
            CorrectionStatus::Failed
        );
        assert_eq!(
            "SKIPPED".parse::<CorrectionStatus>().unwrap(),
            CorrectionStatus::Skipped
        );
    }

    #[test]
    fn test_correction_status_from_str_invalid() {
        assert!("invalid".parse::<CorrectionStatus>().is_err());
        assert!("pending".parse::<CorrectionStatus>().is_err()); // lowercase
    }

    #[test]
    fn test_correction_status_display() {
        assert_eq!(format!("{}", CorrectionStatus::Pending), "PENDING");
        assert_eq!(format!("{}", CorrectionStatus::Completed), "COMPLETED");
    }

    #[test]
    fn test_correction_status_serde() {
        let status = CorrectionStatus::Completed;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"COMPLETED\"");

        let parsed: CorrectionStatus = serde_json::from_str("\"FAILED\"").unwrap();
        assert_eq!(parsed, CorrectionStatus::Failed);
    }

    // =========================================================================
    // TripPathCorrection Tests
    // =========================================================================

    #[test]
    fn test_trip_path_correction_serialization() {
        let correction = TripPathCorrection {
            id: Uuid::new_v4(),
            trip_id: Uuid::new_v4(),
            original_path: r#"{"type":"LineString","coordinates":[[-120.0,45.0],[-120.1,45.1]]}"#
                .to_string(),
            corrected_path: None,
            correction_quality: None,
            correction_status: CorrectionStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&correction).unwrap();
        assert!(json.contains("original_path"));
        assert!(json.contains("PENDING"));
        // Skip serializing None fields
        assert!(!json.contains("corrected_path"));
        assert!(!json.contains("correction_quality"));
    }

    #[test]
    fn test_trip_path_correction_with_correction() {
        let correction = TripPathCorrection {
            id: Uuid::new_v4(),
            trip_id: Uuid::new_v4(),
            original_path: r#"{"type":"LineString","coordinates":[[-120.0,45.0],[-120.1,45.1]]}"#
                .to_string(),
            corrected_path: Some(
                r#"{"type":"LineString","coordinates":[[-120.0,45.0],[-120.05,45.05],[-120.1,45.1]]}"#
                    .to_string(),
            ),
            correction_quality: Some(0.95),
            correction_status: CorrectionStatus::Completed,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&correction).unwrap();
        assert!(json.contains("corrected_path"));
        assert!(json.contains("0.95"));
        assert!(json.contains("COMPLETED"));
    }

    // =========================================================================
    // TripPathResponse Tests
    // =========================================================================

    #[test]
    fn test_trip_path_response_serialization() {
        let response = TripPathResponse {
            original_path: vec![[45.0, -120.0], [45.1, -120.1]],
            corrected_path: None,
            correction_status: CorrectionStatus::Pending,
            correction_quality: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("original_path"));
        assert!(json.contains("PENDING"));
        assert!(!json.contains("corrected_path"));
    }

    #[test]
    fn test_trip_path_response_with_correction() {
        let response = TripPathResponse {
            original_path: vec![[45.0, -120.0], [45.1, -120.1]],
            corrected_path: Some(vec![[45.0, -120.0], [45.05, -120.05], [45.1, -120.1]]),
            correction_status: CorrectionStatus::Completed,
            correction_quality: Some(0.92),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("corrected_path"));
        assert!(json.contains("0.92"));
        assert!(json.contains("COMPLETED"));
    }

    // =========================================================================
    // CorrectPathResponse Tests
    // =========================================================================

    #[test]
    fn test_correct_path_response_serialization() {
        let response = CorrectPathResponse {
            status: "processing".to_string(),
            message: "Path correction queued".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("processing"));
        assert!(json.contains("Path correction queued"));
    }
}
