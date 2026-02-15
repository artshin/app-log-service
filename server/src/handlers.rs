//! HTTP route handlers for the log server.
//!
//! Implements the REST API endpoints for log management.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        Html, IntoResponse, Response,
    },
    Json,
};
use futures::stream::Stream;
use serde::Deserialize;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tracing::info;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    display,
    models::{LogEntry, LogPollResponse, LogRequest, LogUploadRequest},
    AppState,
};

/// GET / - Serve the React SPA
pub async fn handle_root() -> Response {
    match std::fs::read_to_string("static/app/index.html") {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "SPA not built. Run 'npm run build' in web/",
        )
            .into_response(),
    }
}

/// GET /info - Endpoint documentation in plain text
pub async fn handle_info() -> &'static str {
    r#"Log Server (Rust)
==========

Endpoints:
- GET /              - HTML dashboard (visual log viewer)
- POST /logs         - Submit a log entry
- GET /logs          - Retrieve all logs (JSON)
- DELETE /logs       - Clear all logs

Visit / for the interactive web dashboard, or /logs for JSON API access.

Server is listening on port 9006
"#
}

/// POST /logs - Receive and store a log entry
pub async fn handle_receive_log(
    State(state): State<Arc<AppState>>,
    Json(entry): Json<LogEntry>,
) -> Response {
    // Store in buffer
    state.buffer.append(entry.clone());

    // Display in terminal
    display::display_log(&entry, state.verbose);

    StatusCode::CREATED.into_response()
}

/// GET /logs - Retrieve all logs in chronological order
pub async fn handle_get_all_logs(State(state): State<Arc<AppState>>) -> Json<Vec<LogEntry>> {
    let entries = state.buffer.get_all();
    Json(entries)
}

/// DELETE /logs - Clear all logs
pub async fn handle_clear_logs(State(state): State<Arc<AppState>>) -> StatusCode {
    state.buffer.clear();
    info!("Cleared all logs");
    StatusCode::NO_CONTENT
}

/// GET /stream - Server-Sent Events stream for real-time log updates
pub async fn handle_stream(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    info!("New SSE client connected");

    // Subscribe to broadcast channel
    let receiver = state.buffer.subscribe();

    // Convert broadcast receiver to stream
    let stream = BroadcastStream::new(receiver)
        .filter_map(|result| {
            match result {
                Ok(entry) => {
                    // Serialize log entry to JSON
                    match serde_json::to_string(&entry) {
                        Ok(json) => Some(Ok(Event::default().event("log").data(json))),
                        Err(e) => {
                            tracing::error!("Failed to serialize log entry: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Broadcast receive error: {}", e);
                    None
                }
            }
        });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

// MARK: - Protected Endpoints (Require JWT Authentication)

/// Request body for creating a log request
#[derive(Deserialize)]
pub struct CreateRequestBody {
    #[serde(rename = "device_id")]
    pub device_id: String,
}

/// POST /logs/request - Create a log request for a specific device (Admin/Server)
pub async fn handle_create_request(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(body): Json<CreateRequestBody>,
) -> Result<Json<LogRequest>, (StatusCode, String)> {
    let request = state
        .request_manager
        .create_request(auth.user_id, body.device_id.clone());

    info!(
        user_id = %auth.user_id,
        device_id = %body.device_id,
        request_id = %request.id,
        "Log request created"
    );

    Ok(Json(request))
}

/// Query parameters for polling
#[derive(Deserialize)]
pub struct PollQuery {
    #[serde(rename = "deviceId")]
    pub device_id: String,
}

/// GET /logs/poll?deviceId={uuid} - Client polls for pending log requests
pub async fn handle_poll(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(params): Query<PollQuery>,
) -> Result<Json<Option<LogPollResponse>>, (StatusCode, String)> {
    // Check if there's a pending request for this device
    if let Some(request) = state.request_manager.get_pending(&params.device_id) {
        // Verify the request belongs to the authenticated user
        if request.user_id != auth.user_id {
            return Err((
                StatusCode::FORBIDDEN,
                "This log request belongs to a different user".to_string(),
            ));
        }

        let response = LogPollResponse {
            request_id: request.id.to_string(),
            requested_at: request.requested_at.to_rfc3339(),
            expires_at: request.expires_at.to_rfc3339(),
        };

        info!(
            user_id = %auth.user_id,
            device_id = %params.device_id,
            request_id = %request.id,
            "Client polling - pending request found"
        );

        Ok(Json(Some(response)))
    } else {
        // No pending request
        Ok(Json(None))
    }
}

/// POST /logs/upload - Client uploads logs in response to a request
pub async fn handle_upload(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(upload): Json<LogUploadRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Parse request ID
    let request_id = Uuid::parse_str(&upload.request_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "Invalid request ID format".to_string(),
        )
    })?;

    // Verify the request exists and belongs to this user
    // We need to check the request manager to get the request
    let pending = state.request_manager.get_pending(&upload.device_id);

    if let Some(request) = pending {
        if request.id != request_id {
            return Err((
                StatusCode::BAD_REQUEST,
                "Request ID does not match pending request".to_string(),
            ));
        }

        if request.user_id != auth.user_id {
            return Err((
                StatusCode::FORBIDDEN,
                "This log request belongs to a different user".to_string(),
            ));
        }
    } else {
        return Err((
            StatusCode::NOT_FOUND,
            "No pending request found for this device".to_string(),
        ));
    }

    // Save logs to storage
    let _metadata = state
        .storage
        .save_upload(auth.user_id, &upload.device_id, request_id, &upload.logs)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to save logs: {}", e),
            )
        })?;

    // Mark request as fulfilled
    let file_path = format!(
        "{}/{}/{}.jsonl",
        auth.user_id, upload.device_id, request_id
    );
    state.request_manager.fulfill(request_id, file_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fulfill request: {}", e),
        )
    })?;

    info!(
        user_id = %auth.user_id,
        device_id = %upload.device_id,
        request_id = %request_id,
        log_count = upload.total_count,
        "Logs uploaded successfully"
    );

    Ok(StatusCode::CREATED)
}

/// GET /logs/uploads - List all uploaded log files for the authenticated user
pub async fn handle_list_uploads(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Vec<crate::models::LogUploadMetadata>>, (StatusCode, String)> {
    let uploads = state.storage.list_uploads(auth.user_id).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to list uploads: {}", e),
        )
    })?;

    Ok(Json(uploads))
}

/// GET /logs/uploads/:request_id - Download a specific uploaded log file
pub async fn handle_get_upload(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(request_id_str): Path<String>,
) -> Result<Json<Vec<LogEntry>>, (StatusCode, String)> {
    // Parse request ID
    let request_id = Uuid::parse_str(&request_id_str).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "Invalid request ID format".to_string(),
        )
    })?;

    // List all uploads to find the device_id for this request
    let uploads = state.storage.list_uploads(auth.user_id).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to list uploads: {}", e),
        )
    })?;

    let device_id = uploads
        .iter()
        .find(|u| u.request_id == request_id_str)
        .map(|u| u.device_id.clone())
        .ok_or((StatusCode::NOT_FOUND, "Upload not found".to_string()))?;

    // Read logs from storage
    let logs = state
        .storage
        .read_upload(auth.user_id, &device_id, request_id)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read logs: {}", e),
            )
        })?;

    Ok(Json(logs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_info_handler() {
        let response = handle_info().await;
        assert!(response.contains("Log Server (Rust)"));
        assert!(response.contains("POST /logs"));
        assert!(response.contains("GET /logs"));
        assert!(response.contains("DELETE /logs"));
        assert!(response.contains("HTML dashboard"));
    }
}
