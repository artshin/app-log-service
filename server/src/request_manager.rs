//! Log request lifecycle management.
//!
//! Manages pending log requests from server to clients, with automatic expiration.

use crate::models::{LogRequest, LogRequestStatus};
use chrono::{Duration, Utc};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Manages log requests with in-memory storage
#[derive(Clone)]
pub struct RequestManager {
    /// Active requests keyed by device_id
    requests: Arc<RwLock<HashMap<String, LogRequest>>>,
}

impl RequestManager {
    /// Create a new request manager
    pub fn new() -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new log request for a specific device
    ///
    /// If a pending request already exists for this device, it will be cancelled
    /// and replaced with the new request.
    pub fn create_request(&self, user_id: Uuid, device_id: String) -> LogRequest {
        let now = Utc::now();
        let expires_at = now + Duration::hours(24);

        let request = LogRequest {
            id: Uuid::new_v4(),
            user_id,
            device_id: device_id.clone(),
            requested_at: now,
            expires_at,
            status: LogRequestStatus::Pending,
            fulfilled_at: None,
            log_file_path: None,
        };

        let mut requests = self.requests.write();

        // Cancel any existing pending request for this device
        if let Some(existing) = requests.get(&device_id) {
            if existing.status == LogRequestStatus::Pending {
                tracing::info!(
                    device_id = %device_id,
                    old_request_id = %existing.id,
                    new_request_id = %request.id,
                    "Replacing existing pending request"
                );
            }
        }

        requests.insert(device_id, request.clone());
        request
    }

    /// Get a pending request for a specific device
    ///
    /// Returns None if no pending request exists or if the request has expired.
    pub fn get_pending(&self, device_id: &str) -> Option<LogRequest> {
        let mut requests = self.requests.write();

        // Clone the request first to avoid borrow issues
        let request = requests.get(device_id).cloned()?;

        // Check if expired
        if request.status == LogRequestStatus::Pending && Utc::now() > request.expires_at {
            // Mark as expired
            let mut expired_request = request.clone();
            expired_request.status = LogRequestStatus::Expired;
            requests.insert(device_id.to_string(), expired_request);

            tracing::info!(
                device_id = %device_id,
                request_id = %request.id,
                "Request expired"
            );

            return None;
        }

        // Return if still pending
        if request.status == LogRequestStatus::Pending {
            Some(request)
        } else {
            None
        }
    }

    /// Mark a request as fulfilled with the uploaded log file path
    pub fn fulfill(&self, request_id: Uuid, file_path: String) -> Result<(), RequestError> {
        let mut requests = self.requests.write();

        // Find the request by ID
        let device_id = requests
            .iter()
            .find(|(_, req)| req.id == request_id)
            .map(|(did, _)| did.clone())
            .ok_or(RequestError::NotFound)?;

        let request = requests
            .get_mut(&device_id)
            .ok_or(RequestError::NotFound)?;

        // Verify it's still pending
        if request.status != LogRequestStatus::Pending {
            return Err(RequestError::AlreadyProcessed);
        }

        // Mark as fulfilled
        request.status = LogRequestStatus::Fulfilled;
        request.fulfilled_at = Some(Utc::now());
        request.log_file_path = Some(file_path.clone());

        tracing::info!(
            device_id = %device_id,
            request_id = %request_id,
            file_path = %file_path,
            "Request fulfilled"
        );

        Ok(())
    }

    /// Cancel a pending request
    pub fn cancel(&self, device_id: &str) -> Result<(), RequestError> {
        let mut requests = self.requests.write();

        let request = requests
            .get_mut(device_id)
            .ok_or(RequestError::NotFound)?;

        if request.status != LogRequestStatus::Pending {
            return Err(RequestError::AlreadyProcessed);
        }

        request.status = LogRequestStatus::Cancelled;

        tracing::info!(
            device_id = %device_id,
            request_id = %request.id,
            "Request cancelled"
        );

        Ok(())
    }

    /// Clean up expired requests
    ///
    /// Should be called periodically (e.g., every hour) to remove old entries.
    pub fn cleanup_expired(&self) -> usize {
        let mut requests = self.requests.write();
        let now = Utc::now();
        let initial_count = requests.len();

        // Mark expired pending requests
        for (device_id, request) in requests.iter_mut() {
            if request.status == LogRequestStatus::Pending && now > request.expires_at {
                tracing::info!(
                    device_id = %device_id,
                    request_id = %request.id,
                    "Marking request as expired during cleanup"
                );
                request.status = LogRequestStatus::Expired;
            }
        }

        // Remove non-pending requests older than 7 days
        let cutoff = now - Duration::days(7);
        requests.retain(|device_id, request| {
            let should_keep = request.status == LogRequestStatus::Pending
                || request.requested_at > cutoff;

            if !should_keep {
                tracing::debug!(
                    device_id = %device_id,
                    request_id = %request.id,
                    status = ?request.status,
                    "Removing old request"
                );
            }

            should_keep
        });

        let removed = initial_count - requests.len();
        if removed > 0 {
            tracing::info!(removed = removed, "Cleaned up old requests");
        }

        removed
    }

    /// Get statistics about active requests
    pub fn stats(&self) -> RequestStats {
        let requests = self.requests.read();
        let now = Utc::now();

        let mut stats = RequestStats::default();

        for request in requests.values() {
            stats.total += 1;

            match request.status {
                LogRequestStatus::Pending => {
                    if now > request.expires_at {
                        stats.expired += 1;
                    } else {
                        stats.pending += 1;
                    }
                }
                LogRequestStatus::Fulfilled => stats.fulfilled += 1,
                LogRequestStatus::Expired => stats.expired += 1,
                LogRequestStatus::Cancelled => stats.cancelled += 1,
            }
        }

        stats
    }
}

impl Default for RequestManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Request manager errors
#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error("Request not found")]
    NotFound,

    #[error("Request already processed")]
    AlreadyProcessed,
}

/// Statistics about active requests
#[derive(Debug, Default)]
pub struct RequestStats {
    pub total: usize,
    pub pending: usize,
    pub fulfilled: usize,
    pub expired: usize,
    pub cancelled: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_get_request() {
        let manager = RequestManager::new();
        let user_id = Uuid::new_v4();
        let device_id = "test-device".to_string();

        let request = manager.create_request(user_id, device_id.clone());
        assert_eq!(request.status, LogRequestStatus::Pending);

        let retrieved = manager.get_pending(&device_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, request.id);
    }

    #[test]
    fn test_fulfill_request() {
        let manager = RequestManager::new();
        let user_id = Uuid::new_v4();
        let device_id = "test-device".to_string();

        let request = manager.create_request(user_id, device_id.clone());
        let result = manager.fulfill(request.id, "/path/to/logs.jsonl".to_string());

        assert!(result.is_ok());
        assert!(manager.get_pending(&device_id).is_none());
    }

    #[test]
    fn test_replace_pending_request() {
        let manager = RequestManager::new();
        let user_id = Uuid::new_v4();
        let device_id = "test-device".to_string();

        let request1 = manager.create_request(user_id, device_id.clone());
        let request2 = manager.create_request(user_id, device_id.clone());

        assert_ne!(request1.id, request2.id);

        let retrieved = manager.get_pending(&device_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, request2.id);
    }
}
