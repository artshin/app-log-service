//! Thread-safe circular buffer for storing log entries.
//!
//! Uses parking_lot::RwLock for better performance than std::sync::RwLock.

use parking_lot::RwLock;
use tokio::sync::broadcast;

use crate::models::{LogEntry, LogLevel};

/// Thread-safe circular buffer for log entries
pub struct LogBuffer {
    inner: RwLock<BufferInner>,
    broadcast_tx: broadcast::Sender<LogEntry>,
}

struct BufferInner {
    entries: Vec<LogEntry>,
    capacity: usize,
    start_index: usize,
    count: usize,
    min_level: LogLevel,
    source_filter: Option<Vec<String>>,
}

impl LogBuffer {
    /// Create a new circular buffer with the given capacity
    pub fn new(capacity: usize) -> Self {
        // Create broadcast channel with a small buffer (100 entries max in memory before old ones are dropped)
        let (broadcast_tx, _) = broadcast::channel(100);

        Self {
            inner: RwLock::new(BufferInner {
                entries: Vec::with_capacity(capacity),
                capacity,
                start_index: 0,
                count: 0,
                min_level: LogLevel::Trace,
                source_filter: None,
            }),
            broadcast_tx,
        }
    }

    /// Subscribe to new log entry notifications
    pub fn subscribe(&self) -> broadcast::Receiver<LogEntry> {
        self.broadcast_tx.subscribe()
    }

    /// Append a log entry to the buffer
    pub fn append(&self, entry: LogEntry) {
        let mut inner = self.inner.write();

        if inner.count < inner.capacity {
            // Buffer not full yet
            inner.entries.push(entry.clone());
            inner.count += 1;
        } else {
            // Overwrite oldest entry
            let idx = inner.start_index;
            let cap = inner.capacity;
            inner.entries[idx] = entry.clone();
            inner.start_index = (idx + 1) % cap;
        }

        // Release lock before broadcasting to prevent deadlock
        drop(inner);

        // Broadcast to SSE subscribers (ignore errors if no listeners)
        let _ = self.broadcast_tx.send(entry);
    }

    /// Get all entries in chronological order
    pub fn get_all(&self) -> Vec<LogEntry> {
        let inner = self.inner.read();

        if inner.count < inner.capacity {
            // Return entries as-is
            inner.entries.clone()
        } else {
            // Reconstruct chronological order
            let mut result = Vec::with_capacity(inner.capacity);
            let tail = &inner.entries[inner.start_index..];
            let head = &inner.entries[..inner.start_index];
            result.extend(tail.iter().cloned());
            result.extend(head.iter().cloned());
            result
        }
    }

    /// Get entries matching current filters
    #[allow(dead_code)]
    pub fn get_filtered(&self) -> Vec<LogEntry> {
        let inner = self.inner.read();
        let all_entries = get_all_from_inner(&inner);

        all_entries
            .into_iter()
            .filter(|entry| {
                // Level filter
                let entry_level = LogLevel::from_str(&entry.level);
                if entry_level < inner.min_level {
                    return false;
                }

                // Source filter
                if let Some(ref sources) = inner.source_filter {
                    if !sources.contains(&entry.source) {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// Set minimum log level filter
    #[allow(dead_code)]
    pub fn set_minimum_level(&self, level: LogLevel) {
        let mut inner = self.inner.write();
        inner.min_level = level;
    }

    /// Set source filter (None = show all)
    #[allow(dead_code)]
    pub fn set_source_filter(&self, sources: Option<Vec<String>>) {
        let mut inner = self.inner.write();
        inner.source_filter = sources;
    }

    /// Clear all entries from the buffer
    pub fn clear(&self) {
        let mut inner = self.inner.write();
        inner.entries.clear();
        inner.start_index = 0;
        inner.count = 0;
    }

    /// Get current number of entries
    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        let inner = self.inner.read();
        inner.count
    }
}

/// Helper function to get all entries from inner buffer
fn get_all_from_inner(inner: &BufferInner) -> Vec<LogEntry> {
    if inner.count < inner.capacity {
        inner.entries.clone()
    } else {
        let mut result = Vec::with_capacity(inner.capacity);
        let tail = &inner.entries[inner.start_index..];
        let head = &inner.entries[..inner.start_index];
        result.extend(tail.iter().cloned());
        result.extend(head.iter().cloned());
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_entry(id: &str, level: &str) -> LogEntry {
        LogEntry {
            id: id.to_string(),
            timestamp: Utc::now(),
            level: level.to_string(),
            message: format!("Message {}", id),
            user_id: None,
            device_id: "test-device".to_string(),
            source: "test".to_string(),
            metadata: HashMap::new(),
            tags: Vec::new(),
            file: String::new(),
            function: String::new(),
            line: 0,
        }
    }

    #[test]
    fn test_buffer_append_and_get() {
        let buffer = LogBuffer::new(3);

        buffer.append(create_entry("1", "info"));
        buffer.append(create_entry("2", "info"));

        let entries = buffer.get_all();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "1");
        assert_eq!(entries[1].id, "2");
    }

    #[test]
    fn test_buffer_circular_behavior() {
        let buffer = LogBuffer::new(3);

        buffer.append(create_entry("1", "info"));
        buffer.append(create_entry("2", "info"));
        buffer.append(create_entry("3", "info"));
        buffer.append(create_entry("4", "info")); // Should overwrite "1"

        let entries = buffer.get_all();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].id, "2"); // Oldest
        assert_eq!(entries[1].id, "3");
        assert_eq!(entries[2].id, "4"); // Newest
    }

    #[test]
    fn test_buffer_clear() {
        let buffer = LogBuffer::new(10);

        buffer.append(create_entry("1", "info"));
        buffer.append(create_entry("2", "info"));
        assert_eq!(buffer.count(), 2);

        buffer.clear();
        assert_eq!(buffer.count(), 0);
        assert!(buffer.get_all().is_empty());
    }

    #[test]
    fn test_buffer_level_filter() {
        let buffer = LogBuffer::new(10);

        buffer.append(create_entry("1", "debug"));
        buffer.append(create_entry("2", "info"));
        buffer.append(create_entry("3", "error"));

        buffer.set_minimum_level(LogLevel::Info);

        let filtered = buffer.get_filtered();
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, "2");
        assert_eq!(filtered[1].id, "3");
    }
}
