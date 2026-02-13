//! Data models for log entries.
//!
//! Matches the LogEntryDTO structure from Swift's NetworkLogHandler.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Log entry received from Swift clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Unique identifier
    pub id: String,

    /// Timestamp when the log was created
    pub timestamp: DateTime<Utc>,

    /// Log level (trace, debug, info, notice, warning, error, critical)
    pub level: String,

    /// Log message
    pub message: String,

    /// User ID (from JWT token) - optional, only present if authenticated
    #[serde(rename = "userId", skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    /// Persistent device identifier (UUID)
    #[serde(rename = "deviceId")]
    pub device_id: String,

    /// Source identifier (e.g., "cli", "ios", "ios-simulator", "ios-device")
    pub source: String,

    /// Optional metadata key-value pairs
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,

    /// Tags for categorization (client-provided)
    #[serde(default)]
    pub tags: Vec<String>,

    /// Source file path
    #[serde(default)]
    pub file: String,

    /// Function name
    #[serde(default)]
    pub function: String,

    /// Line number
    #[serde(default)]
    pub line: u32,
}

/// Log severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Notice = 3,
    Warning = 4,
    Error = 5,
    Critical = 6,
}

impl LogLevel {
    /// Parse log level from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "trace" => Self::Trace,
            "debug" => Self::Debug,
            "info" => Self::Info,
            "notice" => Self::Notice,
            "warning" => Self::Warning,
            "error" => Self::Error,
            "critical" => Self::Critical,
            _ => Self::Info,
        }
    }

    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Notice => "notice",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Critical => "critical",
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Trace < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Notice);
        assert!(LogLevel::Notice < LogLevel::Warning);
        assert!(LogLevel::Warning < LogLevel::Error);
        assert!(LogLevel::Error < LogLevel::Critical);
    }

    #[test]
    fn test_log_level_parsing() {
        assert_eq!(LogLevel::from_str("trace"), LogLevel::Trace);
        assert_eq!(LogLevel::from_str("DEBUG"), LogLevel::Debug);
        assert_eq!(LogLevel::from_str("Info"), LogLevel::Info);
        assert_eq!(LogLevel::from_str("unknown"), LogLevel::Info); // default
    }

    #[test]
    fn test_log_entry_deserialization() {
        let json = r#"{
            "id": "test-123",
            "timestamp": "2024-01-15T10:30:00Z",
            "level": "info",
            "message": "Test message",
            "deviceId": "device-uuid-123",
            "source": "cli",
            "file": "main.swift",
            "function": "main()",
            "line": 42
        }"#;

        let entry: LogEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.id, "test-123");
        assert_eq!(entry.level, "info");
        assert_eq!(entry.device_id, "device-uuid-123");
        assert_eq!(entry.line, 42);
    }
}

// MARK: - Log Request Models

/// Status of a log request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogRequestStatus {
    /// Request is pending, waiting for client to upload logs
    Pending,
    /// Request has been fulfilled, logs uploaded
    Fulfilled,
    /// Request has expired (24 hours passed)
    Expired,
    /// Request was cancelled
    Cancelled,
}

/// Log request stored on the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRequest {
    /// Unique request identifier
    pub id: Uuid,

    /// User ID who owns the device
    pub user_id: Uuid,

    /// Device identifier to request logs from
    pub device_id: String,

    /// When the request was created
    pub requested_at: DateTime<Utc>,

    /// When the request expires (typically 24 hours after creation)
    pub expires_at: DateTime<Utc>,

    /// Current status of the request
    pub status: LogRequestStatus,

    /// When the request was fulfilled (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfilled_at: Option<DateTime<Utc>>,

    /// Path to the uploaded log file (if fulfilled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_file_path: Option<String>,
}

/// Response sent to client when polling for pending log requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPollResponse {
    /// Request ID
    #[serde(rename = "requestId")]
    pub request_id: String,

    /// When the request was created
    #[serde(rename = "requestedAt")]
    pub requested_at: String,

    /// When the request expires
    #[serde(rename = "expiresAt")]
    pub expires_at: String,
}

/// Request body for uploading logs from client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogUploadRequest {
    /// Request ID this upload fulfills
    #[serde(rename = "requestId")]
    pub request_id: String,

    /// Device ID
    #[serde(rename = "deviceId")]
    pub device_id: String,

    /// Array of log entries
    pub logs: Vec<LogEntry>,

    /// Timestamp of earliest log in the upload
    #[serde(rename = "fromTimestamp")]
    pub from_timestamp: String,

    /// Timestamp of latest log in the upload
    #[serde(rename = "toTimestamp")]
    pub to_timestamp: String,

    /// Total number of logs in the upload
    #[serde(rename = "totalCount")]
    pub total_count: usize,
}

/// Metadata about an uploaded log file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogUploadMetadata {
    /// Request ID
    #[serde(rename = "requestId")]
    pub request_id: String,

    /// Device ID
    #[serde(rename = "deviceId")]
    pub device_id: String,

    /// When the upload was received
    #[serde(rename = "uploadedAt")]
    pub uploaded_at: String,

    /// Number of log entries
    #[serde(rename = "logCount")]
    pub log_count: usize,

    /// File size in bytes
    #[serde(rename = "fileSizeBytes")]
    pub file_size_bytes: u64,
}
