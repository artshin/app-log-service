//! Terminal display for log entries with ANSI colors.
//!
//! Provides colored output matching the Go implementation.

use std::path::Path;

use colored::Colorize;

use crate::models::LogEntry;

/// Display a log entry in the terminal with color coding
pub fn display_log(entry: &LogEntry, verbose: bool) {
    let timestamp = format_timestamp(&entry.timestamp);
    let level_colored = colorize_level(&entry.level);
    let source_label = format_source(&entry.source);

    if verbose {
        // Verbose: [timestamp] LEVEL [source] [file:line] message
        let location = format_location(&entry.file, entry.line);
        println!(
            "{} {} {} {} {}",
            timestamp, level_colored, source_label, location, entry.message
        );

        // Print metadata if present
        if !entry.metadata.is_empty() {
            for (key, value) in &entry.metadata {
                println!("{}", format!("  {}={}", key, value).bright_black());
            }
        }
    } else {
        // Compact: [timestamp] LEVEL [source] message
        println!(
            "{} {} {} {}",
            timestamp, level_colored, source_label, entry.message
        );
    }
}

/// Format timestamp as [HH:MM:SS.mmm] in local time
fn format_timestamp(timestamp: &chrono::DateTime<chrono::Utc>) -> String {
    let local_time = timestamp.with_timezone(&chrono::Local);
    format!("[{}]", local_time.format("%H:%M:%S%.3f"))
}

/// Format source label with cyan color
fn format_source(source: &str) -> String {
    format!("[{}]", source).cyan().to_string()
}

/// Format file and line number with gray color
fn format_location(file: &str, line: u32) -> String {
    let filename = Path::new(file)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(file);
    format!("[{}:{}]", filename, line)
        .bright_black()
        .to_string()
}

/// Colorize log level based on severity
fn colorize_level(level: &str) -> String {
    let level_upper = level.to_uppercase();
    match level.to_lowercase().as_str() {
        "trace" | "debug" => level_upper.bright_black().to_string(),
        "info" => level_upper.green().to_string(),
        "notice" => level_upper.blue().to_string(),
        "warning" => level_upper.yellow().to_string(),
        "error" => level_upper.red().to_string(),
        "critical" => level_upper.magenta().to_string(),
        _ => level_upper.normal().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_location() {
        let location = format_location("/path/to/file.swift", 42);
        // The location should contain the filename and line
        assert!(location.contains("file.swift"));
        assert!(location.contains("42"));
    }

    #[test]
    fn test_format_source() {
        let source = format_source("cli");
        assert!(source.contains("cli"));
    }
}
