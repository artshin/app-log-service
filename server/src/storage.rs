//! File storage for uploaded client logs.
//!
//! Manages persistent storage of log uploads with automatic cleanup.

use crate::models::{LogEntry, LogUploadMetadata};
use chrono::Utc;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use uuid::Uuid;

/// Manages file storage for uploaded logs
#[derive(Clone)]
pub struct LogStorage {
    base_path: PathBuf,
}

impl LogStorage {
    /// Create a new log storage manager
    pub fn new(base_path: PathBuf) -> Result<Self, StorageError> {
        // Create base directory if it doesn't exist
        fs::create_dir_all(&base_path).map_err(|e| {
            StorageError::IoError(format!("Failed to create storage directory: {}", e))
        })?;

        tracing::info!(path = %base_path.display(), "Log storage initialized");

        Ok(Self { base_path })
    }

    /// Save uploaded logs to disk
    ///
    /// Stores logs in: `{base_path}/{user_id}/{device_id}/{request_id}.jsonl`
    pub fn save_upload(
        &self,
        user_id: Uuid,
        device_id: &str,
        request_id: Uuid,
        logs: &[LogEntry],
    ) -> Result<LogUploadMetadata, StorageError> {
        // Sanitize device_id to prevent path traversal
        let safe_device_id = sanitize_filename(device_id);

        // Create directory structure: base/user_id/device_id/
        let user_dir = self.base_path.join(user_id.to_string());
        let device_dir = user_dir.join(&safe_device_id);
        fs::create_dir_all(&device_dir).map_err(|e| {
            StorageError::IoError(format!("Failed to create device directory: {}", e))
        })?;

        // Create log file: request_id.jsonl
        let file_path = device_dir.join(format!("{}.jsonl", request_id));
        let file = File::create(&file_path).map_err(|e| {
            StorageError::IoError(format!("Failed to create log file: {}", e))
        })?;

        let mut writer = BufWriter::new(file);

        // Write logs in JSON Lines format (one JSON object per line)
        for log in logs {
            let json = serde_json::to_string(log).map_err(|e| {
                StorageError::SerializationError(format!("Failed to serialize log entry: {}", e))
            })?;

            writeln!(writer, "{}", json).map_err(|e| {
                StorageError::IoError(format!("Failed to write log entry: {}", e))
            })?;
        }

        writer.flush().map_err(|e| {
            StorageError::IoError(format!("Failed to flush writer: {}", e))
        })?;

        // Get file size
        let metadata = fs::metadata(&file_path).map_err(|e| {
            StorageError::IoError(format!("Failed to read file metadata: {}", e))
        })?;

        let upload_metadata = LogUploadMetadata {
            request_id: request_id.to_string(),
            device_id: device_id.to_string(),
            uploaded_at: Utc::now().to_rfc3339(),
            log_count: logs.len(),
            file_size_bytes: metadata.len(),
        };

        tracing::info!(
            user_id = %user_id,
            device_id = %device_id,
            request_id = %request_id,
            log_count = logs.len(),
            file_size = metadata.len(),
            "Logs saved successfully"
        );

        Ok(upload_metadata)
    }

    /// Read uploaded logs from disk
    pub fn read_upload(
        &self,
        user_id: Uuid,
        device_id: &str,
        request_id: Uuid,
    ) -> Result<Vec<LogEntry>, StorageError> {
        let safe_device_id = sanitize_filename(device_id);
        let file_path = self
            .base_path
            .join(user_id.to_string())
            .join(&safe_device_id)
            .join(format!("{}.jsonl", request_id));

        if !file_path.exists() {
            return Err(StorageError::NotFound);
        }

        let content = fs::read_to_string(&file_path).map_err(|e| {
            StorageError::IoError(format!("Failed to read log file: {}", e))
        })?;

        let mut logs = Vec::new();

        // Parse JSON Lines format
        for (line_num, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            let log: LogEntry = serde_json::from_str(line).map_err(|e| {
                StorageError::SerializationError(format!(
                    "Failed to parse log entry at line {}: {}",
                    line_num + 1,
                    e
                ))
            })?;

            logs.push(log);
        }

        Ok(logs)
    }

    /// List all uploads for a specific user
    pub fn list_uploads(&self, user_id: Uuid) -> Result<Vec<LogUploadMetadata>, StorageError> {
        let user_dir = self.base_path.join(user_id.to_string());

        if !user_dir.exists() {
            return Ok(Vec::new());
        }

        let mut uploads = Vec::new();

        // Iterate through device directories
        let device_dirs = fs::read_dir(&user_dir).map_err(|e| {
            StorageError::IoError(format!("Failed to read user directory: {}", e))
        })?;

        for device_entry in device_dirs {
            let device_entry = device_entry.map_err(|e| {
                StorageError::IoError(format!("Failed to read device entry: {}", e))
            })?;

            if !device_entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                continue;
            }

            let device_id = device_entry.file_name().to_string_lossy().to_string();

            // Iterate through log files in device directory
            let log_files = fs::read_dir(device_entry.path()).map_err(|e| {
                StorageError::IoError(format!("Failed to read device directory: {}", e))
            })?;

            for file_entry in log_files {
                let file_entry = file_entry.map_err(|e| {
                    StorageError::IoError(format!("Failed to read file entry: {}", e))
                })?;

                if !file_entry
                    .file_type()
                    .map(|ft| ft.is_file())
                    .unwrap_or(false)
                {
                    continue;
                }

                let file_name = file_entry.file_name().to_string_lossy().to_string();
                if !file_name.ends_with(".jsonl") {
                    continue;
                }

                let request_id = file_name.trim_end_matches(".jsonl").to_string();

                let metadata = fs::metadata(file_entry.path()).map_err(|e| {
                    StorageError::IoError(format!("Failed to read file metadata: {}", e))
                })?;

                // Count lines in file
                let content = fs::read_to_string(file_entry.path()).map_err(|e| {
                    StorageError::IoError(format!("Failed to read file: {}", e))
                })?;
                let log_count = content.lines().filter(|l| !l.trim().is_empty()).count();

                uploads.push(LogUploadMetadata {
                    request_id,
                    device_id: device_id.clone(),
                    uploaded_at: metadata
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| {
                            chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                                .unwrap_or_default()
                                .to_rfc3339()
                        })
                        .unwrap_or_else(|| Utc::now().to_rfc3339()),
                    log_count,
                    file_size_bytes: metadata.len(),
                });
            }
        }

        Ok(uploads)
    }

    /// Delete old log files (cleanup)
    ///
    /// Removes files older than the specified number of days.
    pub fn cleanup_old_logs(&self, days: i64) -> Result<usize, StorageError> {
        let cutoff = Utc::now() - chrono::Duration::days(days);
        let cutoff_timestamp = cutoff.timestamp();

        let mut removed = 0;

        // Iterate through all user directories
        let user_dirs = fs::read_dir(&self.base_path).map_err(|e| {
            StorageError::IoError(format!("Failed to read base directory: {}", e))
        })?;

        for user_entry in user_dirs {
            let user_entry = match user_entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            if !user_entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                continue;
            }

            // Iterate through device directories
            let device_dirs = match fs::read_dir(user_entry.path()) {
                Ok(dirs) => dirs,
                Err(_) => continue,
            };

            for device_entry in device_dirs {
                let device_entry = match device_entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                if !device_entry
                    .file_type()
                    .map(|ft| ft.is_dir())
                    .unwrap_or(false)
                {
                    continue;
                }

                // Iterate through log files
                let log_files = match fs::read_dir(device_entry.path()) {
                    Ok(files) => files,
                    Err(_) => continue,
                };

                for file_entry in log_files {
                    let file_entry = match file_entry {
                        Ok(e) => e,
                        Err(_) => continue,
                    };

                    if !file_entry
                        .file_type()
                        .map(|ft| ft.is_file())
                        .unwrap_or(false)
                    {
                        continue;
                    }

                    // Check file age
                    let metadata = match fs::metadata(file_entry.path()) {
                        Ok(m) => m,
                        Err(_) => continue,
                    };

                    let modified = match metadata.modified() {
                        Ok(t) => t,
                        Err(_) => continue,
                    };

                    let timestamp = match modified.duration_since(std::time::UNIX_EPOCH) {
                        Ok(d) => d.as_secs() as i64,
                        Err(_) => continue,
                    };

                    if timestamp < cutoff_timestamp {
                        if fs::remove_file(file_entry.path()).is_ok() {
                            removed += 1;
                            tracing::debug!(
                                path = %file_entry.path().display(),
                                "Removed old log file"
                            );
                        }
                    }
                }
            }
        }

        if removed > 0 {
            tracing::info!(removed = removed, days = days, "Cleaned up old log files");
        }

        Ok(removed)
    }
}

/// Sanitize a filename to prevent path traversal attacks
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}

/// Storage errors
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("File not found")]
    NotFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test-device"), "test-device");
        assert_eq!(sanitize_filename("../../../etc/passwd"), "etcpasswd");
        assert_eq!(sanitize_filename("device@#$%123"), "device123");
    }
}
