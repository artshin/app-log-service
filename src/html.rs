//! HTML dashboard generation for log visualization.
//!
//! Generates a responsive, interactive HTML5 dashboard for viewing buffered log entries.
//! Features server-side rendering with client-side JavaScript for filtering and search.

use crate::models::LogEntry;
use std::collections::HashMap;

/// Generate a complete HTML5 dashboard page displaying all log entries.
///
/// Returns a valid HTML5 document with embedded CSS and JavaScript.
/// Handles empty state gracefully with a friendly message.
///
/// # Arguments
/// * `entries` - Slice of log entries to display
///
/// # Returns
/// A complete HTML string ready to serve to browsers
pub fn generate_dashboard_html(entries: &[LogEntry]) -> String {
    if entries.is_empty() {
        return generate_empty_state();
    }

    let mut html = String::with_capacity(entries.len() * 600);

    html.push_str("<!DOCTYPE html>\n");
    html.push_str("<html lang=\"en\">\n");
    html.push_str("<head>\n");
    html.push_str(&generate_html_head());
    html.push_str("</head>\n");
    html.push_str("<body>\n");
    html.push_str("<div class=\"container\">\n");

    // Header
    html.push_str("<div class=\"header\">\n");
    html.push_str("<h1>Log Dashboard</h1>\n");
    html.push_str("<p class=\"subtitle\">Real-time log viewer for development</p>\n");
    html.push_str("</div>\n");

    // Stats section
    html.push_str(&generate_stats_section(entries));

    // Controls section
    html.push_str(&generate_controls_section(entries));

    // Main table
    html.push_str(&generate_log_table(entries));

    // Footer
    html.push_str("<div class=\"footer\">\n");
    html.push_str("<p>View raw JSON: <a href=\"/logs\" target=\"_blank\">/logs</a> | API docs: <a href=\"/info\" target=\"_blank\">/info</a></p>\n");
    html.push_str("</div>\n");

    html.push_str("</div>\n");
    html.push_str(&generate_javascript());
    html.push_str("</body>\n");
    html.push_str("</html>\n");

    html
}

/// Generate HTML head with embedded CSS
fn generate_html_head() -> String {
    r#"<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Log Dashboard</title>
<style>
    * {
        margin: 0;
        padding: 0;
        box-sizing: border-box;
    }

    html, body {
        height: 100%;
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
        background: #f5f5f5;
        color: #333;
    }

    .container {
        max-width: 1400px;
        margin: 0 auto;
        padding: 20px;
    }

    .header {
        background: white;
        padding: 30px;
        border-radius: 8px;
        margin-bottom: 20px;
        box-shadow: 0 1px 3px rgba(0,0,0,0.1);
    }

    .header h1 {
        font-size: 28px;
        margin-bottom: 8px;
        color: #2c3e50;
    }

    .header .subtitle {
        color: #7f8c8d;
        font-size: 14px;
    }

    .stats {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
        margin-bottom: 12px;
        align-items: center;
    }

    .stat-item {
        display: flex;
        align-items: center;
        gap: 6px;
        background: white;
        padding: 6px 12px;
        border-radius: 6px;
        box-shadow: 0 1px 2px rgba(0,0,0,0.08);
        font-size: 13px;
    }

    .stat-item .label {
        color: #7f8c8d;
        font-size: 11px;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    .stat-item .value {
        font-weight: 600;
        color: #2c3e50;
    }

    .stat-divider {
        width: 1px;
        height: 20px;
        background: #e0e0e0;
        margin: 0 4px;
    }

    .level-pills {
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
    }

    .level-pill {
        display: flex;
        align-items: center;
        gap: 4px;
        padding: 3px 8px;
        border-radius: 4px;
        font-size: 11px;
        font-weight: 600;
    }

    .level-pill .count {
        background: rgba(255,255,255,0.3);
        padding: 1px 5px;
        border-radius: 3px;
        font-size: 10px;
    }

    .level-pill.trace { background: #808080; color: white; }
    .level-pill.debug { background: #808080; color: white; }
    .level-pill.info { background: #00aa00; color: white; }
    .level-pill.notice { background: #0088ff; color: white; }
    .level-pill.warning { background: #ffaa00; color: white; }
    .level-pill.error { background: #ff0000; color: white; }
    .level-pill.critical { background: #ff00ff; color: white; }

    .badge-trace { background: #808080; }
    .badge-debug { background: #808080; }
    .badge-info { background: #00aa00; }
    .badge-notice { background: #0088ff; }
    .badge-warning { background: #ffaa00; }
    .badge-error { background: #ff0000; }
    .badge-critical { background: #ff00ff; }

    .controls {
        background: white;
        padding: 20px;
        border-radius: 8px;
        margin-bottom: 20px;
        box-shadow: 0 1px 3px rgba(0,0,0,0.1);
    }

    .controls h3 {
        font-size: 13px;
        color: #7f8c8d;
        text-transform: uppercase;
        margin-bottom: 15px;
        letter-spacing: 1px;
    }

    .control-group {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: 15px;
        margin-bottom: 15px;
    }

    .control-group:last-child {
        margin-bottom: 0;
    }

    .search-box {
        display: flex;
        gap: 10px;
    }

    .search-box input {
        flex: 1;
        padding: 10px 12px;
        border: 1px solid #ddd;
        border-radius: 4px;
        font-size: 14px;
    }

    .search-box input:focus {
        outline: none;
        border-color: #3498db;
        box-shadow: 0 0 5px rgba(52, 152, 219, 0.3);
    }

    .level-filters {
        display: flex;
        flex-wrap: wrap;
        gap: 10px;
    }

    .level-filters label {
        display: flex;
        align-items: center;
        gap: 6px;
        cursor: pointer;
        user-select: none;
        font-size: 13px;
    }

    .level-filters input[type="checkbox"] {
        cursor: pointer;
        width: 16px;
        height: 16px;
    }

    .source-filter select,
    .refresh-controls select {
        padding: 10px 12px;
        border: 1px solid #ddd;
        border-radius: 4px;
        font-size: 14px;
        cursor: pointer;
    }

    .refresh-controls {
        display: flex;
        gap: 10px;
        align-items: center;
    }

    .refresh-controls label {
        display: flex;
        align-items: center;
        gap: 6px;
        cursor: pointer;
        user-select: none;
        font-size: 13px;
    }

    .refresh-controls input[type="checkbox"] {
        cursor: pointer;
        width: 16px;
        height: 16px;
    }

    .refresh-controls input[type="number"] {
        width: 60px;
        padding: 6px 8px;
        border: 1px solid #ddd;
        border-radius: 4px;
        font-size: 13px;
    }

    .stream-status {
        padding: 4px 10px;
        border-radius: 4px;
        font-size: 12px;
        font-weight: 500;
        margin-left: 10px;
    }

    .stream-status.connected {
        background: #d4edda;
        color: #155724;
    }

    .stream-status.disconnected {
        background: #f8d7da;
        color: #721c24;
    }

    .stream-status.connecting {
        background: #fff3cd;
        color: #856404;
    }

    button {
        padding: 10px 16px;
        border: none;
        border-radius: 4px;
        background: #3498db;
        color: white;
        font-weight: 600;
        cursor: pointer;
        font-size: 13px;
        transition: background 0.2s;
    }

    button:hover {
        background: #2980b9;
    }

    button.secondary {
        background: #95a5a6;
    }

    button.secondary:hover {
        background: #7f8c8d;
    }

    button.danger {
        background: #e74c3c;
    }

    button.danger:hover {
        background: #c0392b;
    }

    button.small {
        padding: 6px 10px;
        font-size: 11px;
    }

    .button-group {
        display: flex;
        gap: 8px;
        flex-wrap: wrap;
        align-items: center;
    }

    .autoscroll-toggle {
        display: flex;
        align-items: center;
        gap: 6px;
        font-size: 12px;
        color: #7f8c8d;
        margin-left: auto;
    }

    .autoscroll-toggle input {
        cursor: pointer;
    }

    .table-container {
        background: white;
        border-radius: 8px;
        box-shadow: 0 1px 3px rgba(0,0,0,0.1);
        margin-bottom: 20px;
        display: flex;
        flex-direction: column;
        flex: 1;
        min-height: 0;
        overflow: hidden;
    }

    .table-header {
        flex-shrink: 0;
    }

    .table-wrapper {
        flex: 1;
        overflow-y: auto;
        overflow-x: auto;
        min-height: 200px;
        max-height: calc(100vh - 320px);
    }

    table {
        width: 100%;
        border-collapse: collapse;
    }

    thead {
        background: #f9f9f9;
        position: sticky;
        top: 0;
        z-index: 10;
    }

    thead::after {
        content: '';
        position: absolute;
        left: 0;
        right: 0;
        bottom: 0;
        height: 2px;
        background: #e0e0e0;
    }

    th {
        padding: 10px 12px;
        text-align: left;
        font-weight: 600;
        font-size: 12px;
        color: #2c3e50;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        background: #f9f9f9;
    }

    th.sortable {
        cursor: pointer;
        user-select: none;
    }

    th.sortable:hover {
        background: #f0f0f0;
    }

    th .sort-icon {
        margin-left: 4px;
        opacity: 0.5;
    }

    th.sorted .sort-icon {
        opacity: 1;
    }

    td {
        padding: 8px 12px;
        border-bottom: 1px solid #e0e0e0;
        font-size: 13px;
    }

    tbody tr {
        transition: background 0.1s;
    }

    tbody tr:hover {
        background: #f9f9f9;
    }

    /* Raw view modal */
    .modal-overlay {
        display: none;
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0,0,0,0.5);
        z-index: 1000;
        justify-content: center;
        align-items: center;
    }

    .modal-overlay.active {
        display: flex;
    }

    .modal {
        background: white;
        border-radius: 8px;
        width: 90%;
        max-width: 900px;
        max-height: 80vh;
        display: flex;
        flex-direction: column;
        box-shadow: 0 4px 20px rgba(0,0,0,0.2);
    }

    .modal-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 16px 20px;
        border-bottom: 1px solid #e0e0e0;
    }

    .modal-header h2 {
        font-size: 18px;
        color: #2c3e50;
        margin: 0;
    }

    .modal-close {
        background: none;
        border: none;
        font-size: 24px;
        cursor: pointer;
        color: #7f8c8d;
        padding: 0;
        line-height: 1;
    }

    .modal-close:hover {
        color: #2c3e50;
    }

    .modal-body {
        flex: 1;
        overflow: auto;
        padding: 20px;
    }

    .modal-actions {
        display: flex;
        gap: 10px;
        padding: 16px 20px;
        border-top: 1px solid #e0e0e0;
        background: #f9f9f9;
    }

    .raw-content {
        font-family: "Monaco", "Courier New", monospace;
        font-size: 12px;
        white-space: pre-wrap;
        word-break: break-all;
        background: #1e1e1e;
        color: #d4d4d4;
        padding: 16px;
        border-radius: 4px;
        max-height: 100%;
        overflow: auto;
    }

    .copy-feedback {
        position: fixed;
        bottom: 20px;
        right: 20px;
        background: #27ae60;
        color: white;
        padding: 12px 20px;
        border-radius: 6px;
        font-size: 14px;
        opacity: 0;
        transform: translateY(10px);
        transition: opacity 0.2s, transform 0.2s;
        z-index: 1001;
    }

    .copy-feedback.show {
        opacity: 1;
        transform: translateY(0);
    }

    .log-row {
        cursor: pointer;
    }

    .log-row.hidden {
        display: none;
    }

    .timestamp {
        color: #7f8c8d;
        font-family: "Monaco", "Courier New", monospace;
        font-size: 12px;
        white-space: nowrap;
    }

    .level-badge {
        padding: 4px 8px;
        border-radius: 3px;
        color: white;
        font-weight: bold;
        font-size: 11px;
        display: inline-block;
        min-width: 55px;
        text-align: center;
        font-family: "Monaco", "Courier New", monospace;
    }

    .source-tag {
        padding: 4px 10px;
        border-radius: 4px;
        background: #ecf0f1;
        color: #2c3e50;
        font-size: 12px;
        font-weight: 500;
        display: inline-block;
        max-width: 200px;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .message {
        font-family: "Monaco", "Courier New", monospace;
        color: #2c3e50;
        word-break: break-word;
        max-width: 500px;
    }

    .detail-row {
        background: #fafafa;
    }

    .detail-row td {
        padding: 20px;
    }

    .details {
        display: flex;
        flex-direction: column;
        gap: 15px;
    }

    .detail-item {
        display: grid;
        grid-template-columns: 80px 1fr;
        gap: 15px;
    }

    .detail-item strong {
        color: #7f8c8d;
        font-size: 12px;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    .detail-item code,
    .detail-item pre {
        background: white;
        border: 1px solid #e0e0e0;
        padding: 10px;
        border-radius: 4px;
        overflow-x: auto;
        font-family: "Monaco", "Courier New", monospace;
        font-size: 12px;
        color: #2c3e50;
    }

    .detail-item pre {
        max-height: 200px;
        overflow-y: auto;
    }

    .expand-icon {
        display: inline-block;
        width: 16px;
        height: 16px;
        margin-right: 8px;
        transition: transform 0.2s;
        font-size: 12px;
        line-height: 16px;
    }

    .log-row.expanded .expand-icon {
        transform: rotate(90deg);
    }

    .empty-state {
        text-align: center;
        padding: 60px 20px;
        background: white;
        border-radius: 8px;
        color: #7f8c8d;
    }

    .empty-state h2 {
        font-size: 24px;
        margin-bottom: 10px;
        color: #2c3e50;
    }

    .empty-state p {
        margin-bottom: 20px;
        font-size: 14px;
    }

    .empty-state button {
        margin-top: 20px;
    }

    .footer {
        text-align: center;
        padding: 20px;
        color: #7f8c8d;
        font-size: 12px;
    }

    .footer a {
        color: #3498db;
        text-decoration: none;
    }

    .footer a:hover {
        text-decoration: underline;
    }

    .no-results {
        text-align: center;
        padding: 40px 20px;
        color: #7f8c8d;
        font-size: 14px;
    }

    .stats-summary {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
        gap: 10px;
        margin-top: 10px;
    }

    .stats-summary-item {
        text-align: center;
        padding: 10px;
        background: white;
        border-radius: 4px;
        border: 1px solid #e0e0e0;
    }

    .stats-summary-item .label {
        font-size: 11px;
        color: #7f8c8d;
        text-transform: uppercase;
    }

    .stats-summary-item .count {
        font-size: 18px;
        font-weight: bold;
        color: #2c3e50;
    }

    @media (max-width: 768px) {
        .container {
            padding: 10px;
        }

        .header {
            padding: 20px;
        }

        .header h1 {
            font-size: 20px;
        }

        .control-group {
            grid-template-columns: 1fr;
        }

        .table-wrapper {
            font-size: 12px;
        }

        th, td {
            padding: 8px 12px;
        }

        .message {
            max-width: 300px;
        }

        .level-filters {
            flex-direction: column;
        }

        .level-filters label {
            display: block;
        }
    }

    @media (max-width: 480px) {
        .container {
            padding: 5px;
        }

        .header {
            padding: 15px;
        }

        .header h1 {
            font-size: 18px;
        }

        th, td {
            padding: 6px 8px;
            font-size: 11px;
        }

        .level-badge {
            font-size: 9px;
            padding: 2px 4px;
        }

        .message {
            max-width: 150px;
            font-size: 11px;
        }

        .stats {
            grid-template-columns: 1fr;
        }
    }
</style>"#.to_string()
}

/// Generate summary statistics section (compact inline format)
fn generate_stats_section(entries: &[LogEntry]) -> String {
    let total = entries.len();

    let mut level_counts: HashMap<&str, usize> = HashMap::new();
    let mut source_counts: HashMap<&str, usize> = HashMap::new();

    for entry in entries {
        *level_counts.entry(entry.level.as_str()).or_insert(0) += 1;
        *source_counts.entry(entry.source.as_str()).or_insert(0) += 1;
    }

    let mut stats_html = String::from("<div class=\"stats\">\n");

    // Total logs
    stats_html.push_str("<div class=\"stat-item\">\n");
    stats_html.push_str("<span class=\"label\">Total</span>\n");
    stats_html.push_str(&format!("<span class=\"value\" id=\"total-count\">{}</span>\n", total));
    stats_html.push_str("</div>\n");

    stats_html.push_str("<div class=\"stat-divider\"></div>\n");

    // Level pills
    stats_html.push_str("<div class=\"level-pills\" id=\"level-counts\">\n");
    let levels = [
        "trace", "debug", "info", "notice", "warning", "error", "critical",
    ];
    for level in &levels {
        let count = level_counts.get(level).unwrap_or(&0);
        if *count > 0 {
            stats_html.push_str(&format!(
                "<span class=\"level-pill {}\" data-level=\"{}\">{}<span class=\"count\">{}</span></span>\n",
                level,
                level,
                level.to_uppercase(),
                count
            ));
        }
    }
    stats_html.push_str("</div>\n");

    // Top sources (compact)
    let mut sources: Vec<_> = source_counts.iter().collect();
    sources.sort_by(|a, b| b.1.cmp(a.1));

    if !sources.is_empty() {
        stats_html.push_str("<div class=\"stat-divider\"></div>\n");
        stats_html.push_str("<div class=\"stat-item\">\n");
        stats_html.push_str("<span class=\"label\">Sources</span>\n");
        let source_summary: Vec<String> = sources
            .iter()
            .take(3)
            .map(|(s, c)| format!("{}({})", escape_html(s), c))
            .collect();
        let extra = if sources.len() > 3 {
            format!(" +{}", sources.len() - 3)
        } else {
            String::new()
        };
        stats_html.push_str(&format!(
            "<span class=\"value\">{}{}</span>\n",
            source_summary.join(", "),
            extra
        ));
        stats_html.push_str("</div>\n");
    }

    stats_html.push_str("</div>\n");
    stats_html
}

/// Generate filter controls section
fn generate_controls_section(entries: &[LogEntry]) -> String {
    let mut sources: Vec<&str> = entries.iter().map(|e| e.source.as_str()).collect();
    sources.sort();
    sources.dedup();

    let mut html = String::from("<div class=\"controls\">\n");

    // First row: Level filters + Source + Search
    html.push_str("<div class=\"control-group\">\n");
    html.push_str("<div class=\"level-filters\">\n");
    let levels = [
        "trace", "debug", "info", "notice", "warning", "error", "critical",
    ];
    for level in &levels {
        html.push_str(&format!(
            "<label><input type=\"checkbox\" class=\"level-filter\" value=\"{}\" checked> {}</label>\n",
            level,
            level.to_uppercase()
        ));
    }
    html.push_str("</div>\n");

    // Source filter
    html.push_str("<div class=\"source-filter\">\n");
    html.push_str("<select id=\"source-select\" aria-label=\"Filter by source\">\n");
    html.push_str("<option value=\"\">All Sources</option>\n");
    for source in sources {
        html.push_str(&format!(
            "<option value=\"{}\">{}</option>\n",
            escape_html(source),
            escape_html(source)
        ));
    }
    html.push_str("</select>\n");
    html.push_str("</div>\n");

    // Search box
    html.push_str("<div class=\"search-box\">\n");
    html.push_str("<input type=\"text\" id=\"search\" placeholder=\"Search messages... (/)\" aria-label=\"Search logs\">\n");
    html.push_str("</div>\n");

    html.push_str("</div>\n");

    // Second row: Buttons + Live streaming
    html.push_str("<div class=\"control-group\">\n");
    html.push_str("<div class=\"button-group\">\n");
    html.push_str("<button id=\"clear-filters\" class=\"secondary small\">Clear Filters</button>\n");
    html.push_str("<button id=\"refresh\" class=\"small\">Refresh</button>\n");
    html.push_str("<button id=\"raw-view\" class=\"secondary small\">Raw JSON</button>\n");
    html.push_str("<button id=\"clear-logs\" class=\"danger small\">Clear All</button>\n");
    html.push_str("</div>\n");

    // Live streaming + sort controls
    html.push_str("<div class=\"refresh-controls\">\n");
    html.push_str("<select id=\"sort-order\" aria-label=\"Sort order\">\n");
    html.push_str("<option value=\"desc\" selected>Newest First</option>\n");
    html.push_str("<option value=\"asc\">Oldest First</option>\n");
    html.push_str("</select>\n");
    html.push_str("<label>\n");
    html.push_str("<input type=\"checkbox\" id=\"live-stream\" checked> Live\n");
    html.push_str("</label>\n");
    html.push_str("<span id=\"stream-status\" class=\"stream-status\">...</span>\n");
    html.push_str("</div>\n");

    html.push_str("</div>\n");
    html.push_str("</div>\n");

    html
}

/// Generate the main log table with all entries
fn generate_log_table(entries: &[LogEntry]) -> String {
    let mut html = String::from("<div class=\"table-container\">\n");

    // Table header bar with autoscroll
    html.push_str("<div class=\"table-header\" style=\"display:flex;justify-content:space-between;align-items:center;padding:8px 12px;border-bottom:1px solid #e0e0e0;background:#fafafa;\">\n");
    html.push_str("<span style=\"font-size:12px;color:#7f8c8d;\">Log Entries</span>\n");
    html.push_str("<label class=\"autoscroll-toggle\"><input type=\"checkbox\" id=\"autoscroll\" checked> Autoscroll</label>\n");
    html.push_str("</div>\n");

    html.push_str("<div class=\"table-wrapper\" id=\"table-wrapper\">\n");
    html.push_str("<table id=\"logs-table\" role=\"table\" aria-label=\"Log entries\">\n");
    html.push_str("<thead>\n");
    html.push_str("<tr>\n");
    html.push_str("<th style=\"width: 150px;\" class=\"sortable sorted\" data-sort=\"timestamp\" onclick=\"toggleSort('timestamp')\">Timestamp <span class=\"sort-icon\">▼</span></th>\n");
    html.push_str("<th style=\"width: 70px;\">Level</th>\n");
    html.push_str("<th style=\"width: 100px;\">Source</th>\n");
    html.push_str("<th>Message</th>\n");
    html.push_str("<th style=\"width: 40px;\"></th>\n");
    html.push_str("</tr>\n");
    html.push_str("</thead>\n");
    html.push_str("<tbody id=\"logs-tbody\">\n");

    // Display entries in reverse order (newest first) by default
    for entry in entries.iter().rev() {
        html.push_str(&generate_log_row(entry));
    }

    html.push_str("</tbody>\n");
    html.push_str("</table>\n");
    html.push_str("</div>\n");
    html.push_str("</div>\n");

    // Raw view modal
    html.push_str(&generate_raw_view_modal());

    // Copy feedback toast
    html.push_str("<div id=\"copy-feedback\" class=\"copy-feedback\">Copied to clipboard!</div>\n");

    html
}

/// Generate the raw view modal
fn generate_raw_view_modal() -> String {
    let mut html = String::new();

    html.push_str("<div id=\"raw-modal\" class=\"modal-overlay\">\n");
    html.push_str("<div class=\"modal\">\n");
    html.push_str("<div class=\"modal-header\">\n");
    html.push_str("<h2>Raw JSON Logs</h2>\n");
    html.push_str("<button class=\"modal-close\" onclick=\"closeRawModal()\">&times;</button>\n");
    html.push_str("</div>\n");
    html.push_str("<div class=\"modal-body\">\n");
    html.push_str("<pre id=\"raw-content\" class=\"raw-content\"></pre>\n");
    html.push_str("</div>\n");
    html.push_str("<div class=\"modal-actions\">\n");
    html.push_str("<button onclick=\"copyAllLogs()\">Copy All</button>\n");
    html.push_str("<button onclick=\"copyFilteredLogs()\" class=\"secondary\">Copy Visible Only</button>\n");
    html.push_str("<button onclick=\"closeRawModal()\" class=\"secondary\">Close</button>\n");
    html.push_str("</div>\n");
    html.push_str("</div>\n");
    html.push_str("</div>\n");

    html
}

/// Generate a single log row with detail row
fn generate_log_row(entry: &LogEntry) -> String {
    let level_class = get_level_badge_class(&entry.level);
    let has_details = !entry.file.is_empty()
        || !entry.function.is_empty()
        || entry.line > 0
        || !entry.metadata.is_empty();

    // Serialize entry to JSON for copy functionality
    let entry_json = serde_json::to_string(entry).unwrap_or_default();
    let entry_json_escaped = escape_html(&entry_json);

    let mut html = String::new();

    html.push_str(&format!(
        "<tr class=\"log-row\" data-level=\"{}\" data-source=\"{}\" data-message=\"{}\" data-id=\"{}\" data-timestamp=\"{}\" data-json=\"{}\">\n",
        escape_html(&entry.level),
        escape_html(&entry.source),
        escape_html(&entry.message),
        escape_html(&entry.id),
        entry.timestamp.timestamp_millis(),
        entry_json_escaped
    ));

    // Timestamp cell
    html.push_str(&format!(
        "<td class=\"timestamp\"{}>{}</td>\n",
        if has_details {
            format!(" style=\"cursor:pointer;\" onclick=\"toggleDetails('{}', event)\"", escape_html(&entry.id))
        } else {
            String::new()
        },
        format_timestamp(&entry.timestamp)
    ));
    if has_details {
        // Add expand icon via CSS ::before or inline
    }

    html.push_str(&format!(
        "<td><span class=\"level-badge badge-{}\">{}</span></td>\n",
        level_class,
        entry.level.to_uppercase()
    ));

    html.push_str(&format!(
        "<td><span class=\"source-tag\" title=\"{}\">{}</span></td>\n",
        escape_html(&entry.source),
        escape_html(&entry.source)
    ));

    let msg_display = if entry.message.len() > 150 {
        format!("{}...", &entry.message[..150])
    } else {
        entry.message.clone()
    };

    html.push_str(&format!(
        "<td class=\"message\" title=\"{}\">{}</td>\n",
        escape_html(&entry.message),
        escape_html(&msg_display)
    ));

    // Copy button
    html.push_str(&format!(
        "<td><button class=\"small secondary\" onclick=\"copyLogEntry('{}', event)\" title=\"Copy JSON\">📋</button></td>\n",
        escape_html(&entry.id)
    ));

    html.push_str("</tr>\n");

    // Detail row
    if has_details {
        html.push_str(&generate_detail_row(entry));
    }

    html
}

/// Generate detail row for expanded information
fn generate_detail_row(entry: &LogEntry) -> String {
    let mut html = String::new();

    html.push_str(&format!(
        "<tr class=\"detail-row\" id=\"detail-{}\" style=\"display:none;\">\n",
        escape_html(&entry.id)
    ));
    html.push_str("<td colspan=\"4\">\n");
    html.push_str("<div class=\"details\">\n");

    // ID
    html.push_str("<div class=\"detail-item\">\n");
    html.push_str("<strong>ID</strong>\n");
    html.push_str(&format!("<code>{}</code>\n", escape_html(&entry.id)));
    html.push_str("</div>\n");

    // File
    if !entry.file.is_empty() {
        html.push_str("<div class=\"detail-item\">\n");
        html.push_str("<strong>File</strong>\n");
        html.push_str(&format!("<code>{}</code>\n", escape_html(&entry.file)));
        html.push_str("</div>\n");
    }

    // Function
    if !entry.function.is_empty() {
        html.push_str("<div class=\"detail-item\">\n");
        html.push_str("<strong>Function</strong>\n");
        html.push_str(&format!("<code>{}</code>\n", escape_html(&entry.function)));
        html.push_str("</div>\n");
    }

    // Line
    if entry.line > 0 {
        html.push_str("<div class=\"detail-item\">\n");
        html.push_str("<strong>Line</strong>\n");
        html.push_str(&format!("<code>{}</code>\n", entry.line));
        html.push_str("</div>\n");
    }

    // Metadata
    if !entry.metadata.is_empty() {
        html.push_str("<div class=\"detail-item\">\n");
        html.push_str("<strong>Metadata</strong>\n");
        html.push_str("<pre>");
        for (key, value) in &entry.metadata {
            html.push_str(&format!("{}: {}\n", escape_html(key), escape_html(value)));
        }
        html.push_str("</pre>\n");
        html.push_str("</div>\n");
    }

    html.push_str("</div>\n");
    html.push_str("</td>\n");
    html.push_str("</tr>\n");

    html
}

/// Get CSS badge class for log level
fn get_level_badge_class(level: &str) -> &'static str {
    match level.to_lowercase().as_str() {
        "trace" => "trace",
        "debug" => "debug",
        "info" => "info",
        "notice" => "notice",
        "warning" => "warning",
        "error" => "error",
        "critical" => "critical",
        _ => "info",
    }
}

/// Format timestamp for display in local time
fn format_timestamp(dt: &chrono::DateTime<chrono::Utc>) -> String {
    let local_time = dt.with_timezone(&chrono::Local);
    local_time.format("%Y-%m-%d %H:%M:%S%.3f %Z").to_string()
}

/// Escape HTML special characters to prevent XSS
pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Generate empty state HTML
fn generate_empty_state() -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Log Dashboard</title>
    {}
</head>
<body>
<div class="container">
    <div class="header">
        <h1>Log Dashboard</h1>
        <p class="subtitle">Real-time log viewer for development</p>
    </div>
    <div class="empty-state">
        <h2>No Logs Yet</h2>
        <p>Waiting for log entries from your application...</p>
        <p><span id="stream-status" class="stream-status">Connecting...</span></p>
        <button onclick="location.reload()">Refresh</button>
    </div>
    <div class="footer">
        <p>View raw JSON: <a href="/logs" target="_blank">/logs</a> | API docs: <a href="/info" target="_blank">/info</a></p>
    </div>
</div>
<script>
// Initialize SSE for empty state
let eventSource = new EventSource('/stream');

eventSource.addEventListener('log', function(event) {{
    // Reload page when first log arrives
    location.reload();
}});

eventSource.onopen = function() {{
    const statusElement = document.getElementById('stream-status');
    statusElement.textContent = 'Connected - Waiting for logs...';
    statusElement.className = 'stream-status connected';
}};

eventSource.onerror = function(error) {{
    const statusElement = document.getElementById('stream-status');
    statusElement.textContent = 'Disconnected';
    statusElement.className = 'stream-status disconnected';
}};
</script>
</body>
</html>"#,
        generate_html_head()
    )
}

/// Generate embedded JavaScript for interactivity
fn generate_javascript() -> String {
    r#"<script>
// Global state
let eventSource = null;
let currentSortOrder = 'desc'; // newest first by default
let allLogsData = [];

// Debounce function
function debounce(func, wait) {
    let timeout;
    return function(...args) {
        clearTimeout(timeout);
        timeout = setTimeout(() => func.apply(this, args), wait);
    };
}

// Collect all log data from rows
function collectLogData() {
    allLogsData = [];
    document.querySelectorAll('.log-row').forEach(row => {
        if (row.dataset.json) {
            try {
                allLogsData.push(JSON.parse(row.dataset.json.replace(/&quot;/g, '"').replace(/&#x27;/g, "'").replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&amp;/g, '&')));
            } catch(e) {}
        }
    });
}

// Filter functions
function filterByLevel() { applyAllFilters(); }
function filterBySource() { applyAllFilters(); }
const debouncedSearch = debounce(() => applyAllFilters(), 300);
function searchLogs() { debouncedSearch(); }

function applyAllFilters() {
    const selectedLevels = Array.from(document.querySelectorAll('.level-filter:checked')).map(cb => cb.value);
    const selectedSource = document.getElementById('source-select').value;
    const searchQuery = document.getElementById('search').value.toLowerCase();

    let visibleCount = 0;
    document.querySelectorAll('.log-row').forEach(row => {
        const level = row.dataset.level;
        const source = row.dataset.source;
        const message = row.dataset.message.toLowerCase();

        let visible = selectedLevels.includes(level);
        if (visible && selectedSource && source !== selectedSource) visible = false;
        if (visible && searchQuery && !message.includes(searchQuery)) visible = false;

        if (visible) visibleCount++;
        row.classList.toggle('hidden', !visible);

        const detailRow = document.getElementById(`detail-${row.dataset.id}`);
        if (detailRow) detailRow.classList.toggle('hidden', !visible);
    });

    updateNoResultsMessage(visibleCount);
}

function toggleDetails(logId, event) {
    event.stopPropagation();
    const detailRow = document.getElementById(`detail-${logId}`);
    const logRow = event.target.closest('.log-row');
    if (!detailRow) return;

    const isVisible = detailRow.style.display !== 'none';
    detailRow.style.display = isVisible ? 'none' : '';
    logRow.classList.toggle('expanded', !isVisible);
}

function clearAllFilters() {
    document.querySelectorAll('.level-filter').forEach(cb => cb.checked = true);
    document.getElementById('source-select').value = '';
    document.getElementById('search').value = '';
    applyAllFilters();
}

function manualRefresh() { location.reload(); }

// Clear all logs
function clearAllLogs() {
    if (!confirm('Are you sure you want to clear all logs?')) return;

    fetch('/logs', { method: 'DELETE' })
        .then(response => {
            if (response.ok) {
                location.reload();
            } else {
                alert('Failed to clear logs');
            }
        })
        .catch(err => alert('Error: ' + err.message));
}

// Sorting
function toggleSort(column) {
    const sortSelect = document.getElementById('sort-order');
    currentSortOrder = currentSortOrder === 'desc' ? 'asc' : 'desc';
    sortSelect.value = currentSortOrder;
    sortTable();
}

function sortTable() {
    const tbody = document.getElementById('logs-tbody');
    const rows = Array.from(tbody.querySelectorAll('.log-row'));

    rows.sort((a, b) => {
        const tsA = parseInt(a.dataset.timestamp) || 0;
        const tsB = parseInt(b.dataset.timestamp) || 0;
        return currentSortOrder === 'desc' ? tsB - tsA : tsA - tsB;
    });

    // Reorder rows with their detail rows
    rows.forEach(row => {
        const detailRow = document.getElementById(`detail-${row.dataset.id}`);
        tbody.appendChild(row);
        if (detailRow) tbody.appendChild(detailRow);
    });

    // Update sort icon
    const th = document.querySelector('th[data-sort="timestamp"]');
    if (th) {
        const icon = th.querySelector('.sort-icon');
        if (icon) icon.textContent = currentSortOrder === 'desc' ? '▼' : '▲';
    }
}

// Raw view modal
function openRawModal() {
    collectLogData();
    const modal = document.getElementById('raw-modal');
    const content = document.getElementById('raw-content');
    content.textContent = JSON.stringify(allLogsData, null, 2);
    modal.classList.add('active');
}

function closeRawModal() {
    document.getElementById('raw-modal').classList.remove('active');
}

function copyAllLogs() {
    collectLogData();
    copyToClipboard(JSON.stringify(allLogsData, null, 2));
}

function copyFilteredLogs() {
    const visibleLogs = [];
    document.querySelectorAll('.log-row:not(.hidden)').forEach(row => {
        if (row.dataset.json) {
            try {
                visibleLogs.push(JSON.parse(row.dataset.json.replace(/&quot;/g, '"').replace(/&#x27;/g, "'").replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&amp;/g, '&')));
            } catch(e) {}
        }
    });
    copyToClipboard(JSON.stringify(visibleLogs, null, 2));
}

function copyLogEntry(logId, event) {
    event.stopPropagation();
    const row = document.querySelector(`.log-row[data-id="${logId}"]`);
    if (row && row.dataset.json) {
        try {
            const json = row.dataset.json.replace(/&quot;/g, '"').replace(/&#x27;/g, "'").replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&amp;/g, '&');
            const parsed = JSON.parse(json);
            copyToClipboard(JSON.stringify(parsed, null, 2));
        } catch(e) {
            console.error('Failed to parse log entry:', e);
        }
    }
}

function copyToClipboard(text) {
    navigator.clipboard.writeText(text).then(() => {
        showCopyFeedback();
    }).catch(err => {
        console.error('Failed to copy:', err);
        // Fallback
        const ta = document.createElement('textarea');
        ta.value = text;
        document.body.appendChild(ta);
        ta.select();
        document.execCommand('copy');
        document.body.removeChild(ta);
        showCopyFeedback();
    });
}

function showCopyFeedback() {
    const feedback = document.getElementById('copy-feedback');
    feedback.classList.add('show');
    setTimeout(() => feedback.classList.remove('show'), 2000);
}

// Autoscroll
function scrollToTop() {
    const wrapper = document.getElementById('table-wrapper');
    if (wrapper) wrapper.scrollTop = 0;
}

// Format timestamp
function formatTimestamp(isoString) {
    const date = new Date(isoString);
    const pad = (n, w = 2) => String(n).padStart(w, '0');
    return `${date.getUTCFullYear()}-${pad(date.getUTCMonth()+1)}-${pad(date.getUTCDate())} ${pad(date.getUTCHours())}:${pad(date.getUTCMinutes())}:${pad(date.getUTCSeconds())}.${pad(date.getUTCMilliseconds(), 3)} UTC`;
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function getLevelBadgeClass(level) {
    const l = level.toLowerCase();
    return ['trace','debug','info','notice','warning','error','critical'].includes(l) ? l : 'info';
}

// Add new log entry (for SSE)
function prependLogEntry(entry) {
    const tbody = document.getElementById('logs-tbody');
    if (!tbody) return;

    const hasDetails = entry.file || entry.function || entry.line > 0 || (entry.metadata && Object.keys(entry.metadata).length > 0);
    const levelClass = getLevelBadgeClass(entry.level);
    const timestamp = formatTimestamp(entry.timestamp);
    const msgDisplay = entry.message.length > 150 ? entry.message.substring(0, 150) + '...' : entry.message;
    const entryJson = escapeHtml(JSON.stringify(entry));

    const tr = document.createElement('tr');
    tr.className = 'log-row';
    tr.dataset.level = entry.level;
    tr.dataset.source = entry.source;
    tr.dataset.message = entry.message;
    tr.dataset.id = entry.id;
    tr.dataset.timestamp = new Date(entry.timestamp).getTime();
    tr.dataset.json = entryJson;

    tr.innerHTML = `
        <td class="timestamp"${hasDetails ? ` style="cursor:pointer;" onclick="toggleDetails('${escapeHtml(entry.id)}', event)"` : ''}>${timestamp}</td>
        <td><span class="level-badge badge-${levelClass}">${entry.level.toUpperCase()}</span></td>
        <td><span class="source-tag" title="${escapeHtml(entry.source)}">${escapeHtml(entry.source)}</span></td>
        <td class="message" title="${escapeHtml(entry.message)}">${escapeHtml(msgDisplay)}</td>
        <td><button class="small secondary" onclick="copyLogEntry('${escapeHtml(entry.id)}', event)" title="Copy JSON">📋</button></td>
    `;

    // Insert based on sort order
    if (currentSortOrder === 'desc') {
        tbody.insertBefore(tr, tbody.firstChild);
    } else {
        tbody.appendChild(tr);
    }

    // Create detail row if needed
    if (hasDetails) {
        const detailTr = document.createElement('tr');
        detailTr.className = 'detail-row';
        detailTr.id = `detail-${entry.id}`;
        detailTr.style.display = 'none';

        let detailsHtml = '<td colspan="5"><div class="details">';
        detailsHtml += `<div class="detail-item"><strong>ID</strong><code>${escapeHtml(entry.id)}</code></div>`;
        if (entry.file) detailsHtml += `<div class="detail-item"><strong>File</strong><code>${escapeHtml(entry.file)}</code></div>`;
        if (entry.function) detailsHtml += `<div class="detail-item"><strong>Function</strong><code>${escapeHtml(entry.function)}</code></div>`;
        if (entry.line > 0) detailsHtml += `<div class="detail-item"><strong>Line</strong><code>${entry.line}</code></div>`;
        if (entry.metadata && Object.keys(entry.metadata).length > 0) {
            detailsHtml += '<div class="detail-item"><strong>Metadata</strong><pre>';
            for (const [key, value] of Object.entries(entry.metadata)) {
                detailsHtml += `${escapeHtml(key)}: ${escapeHtml(value)}\n`;
            }
            detailsHtml += '</pre></div>';
        }
        detailsHtml += '</div></td>';
        detailTr.innerHTML = detailsHtml;

        if (currentSortOrder === 'desc') {
            tbody.insertBefore(detailTr, tr.nextSibling);
        } else {
            tbody.appendChild(detailTr);
        }
    }

    updateStatistics(entry);
    applyAllFilters();

    // Autoscroll if enabled
    if (document.getElementById('autoscroll').checked) {
        scrollToTop();
    }
}

// Update statistics
function updateStatistics(entry) {
    const totalEl = document.getElementById('total-count');
    if (totalEl) totalEl.textContent = parseInt(totalEl.textContent || 0) + 1;

    // Update or create level pill
    const levelCounts = document.getElementById('level-counts');
    if (levelCounts) {
        let pill = levelCounts.querySelector(`.level-pill[data-level="${entry.level.toLowerCase()}"]`);
        if (pill) {
            const countSpan = pill.querySelector('.count');
            if (countSpan) countSpan.textContent = parseInt(countSpan.textContent || 0) + 1;
        } else {
            const newPill = document.createElement('span');
            newPill.className = `level-pill ${entry.level.toLowerCase()}`;
            newPill.dataset.level = entry.level.toLowerCase();
            newPill.innerHTML = `${entry.level.toUpperCase()}<span class="count">1</span>`;
            levelCounts.appendChild(newPill);
        }
    }
}

// SSE connection
function initializeSSE() {
    const statusEl = document.getElementById('stream-status');
    if (eventSource) eventSource.close();

    statusEl.textContent = '...';
    statusEl.className = 'stream-status connecting';

    eventSource = new EventSource('/stream');

    eventSource.addEventListener('log', function(event) {
        try {
            prependLogEntry(JSON.parse(event.data));
        } catch (e) {
            console.error('Failed to parse log entry:', e);
        }
    });

    eventSource.onopen = () => {
        statusEl.textContent = 'Live';
        statusEl.className = 'stream-status connected';
    };

    eventSource.onerror = () => {
        statusEl.textContent = 'Off';
        statusEl.className = 'stream-status disconnected';
        setTimeout(() => {
            if (document.getElementById('live-stream').checked) initializeSSE();
        }, 5000);
    };
}

function closeSSE() {
    if (eventSource) {
        eventSource.close();
        eventSource = null;
        const statusEl = document.getElementById('stream-status');
        statusEl.textContent = 'Off';
        statusEl.className = 'stream-status disconnected';
    }
}

function setupLiveStreaming() {
    const checkbox = document.getElementById('live-stream');
    checkbox.addEventListener('change', function() {
        this.checked ? initializeSSE() : closeSSE();
    });
    if (checkbox.checked) initializeSSE();
}

// Keyboard shortcuts
function setupKeyboardShortcuts() {
    document.addEventListener('keydown', function(event) {
        if (event.key === '/' && event.target.tagName !== 'INPUT') {
            event.preventDefault();
            document.getElementById('search').focus();
        }
        if (event.key === 'Escape') {
            closeRawModal();
        }
    });
}

function updateNoResultsMessage(visibleCount) {
    let msg = document.getElementById('no-results-message');
    if (visibleCount === 0) {
        if (!msg) {
            msg = document.createElement('div');
            msg.id = 'no-results-message';
            msg.className = 'no-results';
            msg.textContent = 'No logs match current filters.';
            document.getElementById('table-wrapper').appendChild(msg);
        }
        msg.style.display = 'block';
    } else if (msg) {
        msg.style.display = 'none';
    }
}

// Initialize
document.addEventListener('DOMContentLoaded', function() {
    document.querySelectorAll('.level-filter').forEach(cb => cb.addEventListener('change', filterByLevel));
    document.getElementById('source-select').addEventListener('change', filterBySource);
    document.getElementById('search').addEventListener('input', searchLogs);
    document.getElementById('clear-filters').addEventListener('click', clearAllFilters);
    document.getElementById('refresh').addEventListener('click', manualRefresh);
    document.getElementById('raw-view').addEventListener('click', openRawModal);
    document.getElementById('clear-logs').addEventListener('click', clearAllLogs);
    document.getElementById('sort-order').addEventListener('change', function() {
        currentSortOrder = this.value;
        sortTable();
    });

    // Close modal on overlay click
    document.getElementById('raw-modal').addEventListener('click', function(e) {
        if (e.target === this) closeRawModal();
    });

    setupLiveStreaming();
    setupKeyboardShortcuts();
    collectLogData();
});
</script>"#
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_entry(id: &str, level: &str, message: &str, source: &str) -> LogEntry {
        LogEntry {
            id: id.to_string(),
            timestamp: Utc::now(),
            level: level.to_string(),
            message: message.to_string(),
            source: source.to_string(),
            metadata: HashMap::new(),
            file: String::new(),
            function: String::new(),
            line: 0,
        }
    }

    #[test]
    fn test_generate_empty_dashboard() {
        let html = generate_dashboard_html(&[]);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("No Logs Yet"));
        assert!(html.contains("Log Dashboard"));
    }

    #[test]
    fn test_generate_single_entry_dashboard() {
        let entry = create_test_entry("1", "info", "Test message", "test-source");
        let html = generate_dashboard_html(&[entry]);
        assert!(html.contains("Test message"));
        assert!(html.contains("test-source"));
        assert!(html.contains("INFO"));
    }

    #[test]
    fn test_html_escaping_xss() {
        let entry = create_test_entry("1", "error", "<script>alert('xss')</script>", "<img src=x>");
        let html = generate_dashboard_html(&[entry]);
        assert!(!html.contains("<script>alert"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<img src=x>"));
        assert!(html.contains("&lt;img"));
    }

    #[test]
    fn test_level_badge_class() {
        assert_eq!(get_level_badge_class("trace"), "trace");
        assert_eq!(get_level_badge_class("debug"), "debug");
        assert_eq!(get_level_badge_class("info"), "info");
        assert_eq!(get_level_badge_class("notice"), "notice");
        assert_eq!(get_level_badge_class("warning"), "warning");
        assert_eq!(get_level_badge_class("error"), "error");
        assert_eq!(get_level_badge_class("critical"), "critical");
        assert_eq!(get_level_badge_class("unknown"), "info");
    }

    #[test]
    fn test_escape_html_function() {
        assert_eq!(escape_html("&"), "&amp;");
        assert_eq!(escape_html("<"), "&lt;");
        assert_eq!(escape_html(">"), "&gt;");
        assert_eq!(escape_html("\""), "&quot;");
        assert_eq!(escape_html("'"), "&#x27;");
        assert_eq!(
            escape_html("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"
        );
    }

    #[test]
    fn test_stats_section_generation() {
        let entries = vec![
            create_test_entry("1", "info", "msg1", "source1"),
            create_test_entry("2", "info", "msg2", "source1"),
            create_test_entry("3", "error", "msg3", "source2"),
        ];

        let stats_html = generate_stats_section(&entries);
        assert!(stats_html.contains("3")); // total
        assert!(stats_html.contains("2")); // info count
        assert!(stats_html.contains("1")); // error count
    }

    #[test]
    fn test_timestamp_formatting() {
        let dt = Utc::now();
        let formatted = format_timestamp(&dt);
        assert!(!formatted.is_empty());
        assert!(formatted.contains("UTC"));
    }

    #[test]
    fn test_multiple_entries_with_details() {
        let mut entry1 = create_test_entry("1", "info", "Test message 1", "source1");
        entry1.file = "/path/to/file.swift".to_string();
        entry1.function = "main()".to_string();
        entry1.line = 42;

        let entry2 = create_test_entry("2", "error", "Test message 2", "source2");

        let html = generate_dashboard_html(&[entry1, entry2]);
        assert!(html.contains("Test message 1"));
        assert!(html.contains("Test message 2"));
        assert!(html.contains("file.swift"));
        assert!(html.contains("main()"));
        assert!(html.contains("42"));
    }

    #[test]
    fn test_dashboard_contains_javascript() {
        let entry = create_test_entry("1", "info", "msg", "src");
        let html = generate_dashboard_html(&[entry]);
        assert!(html.contains("<script>"));
        assert!(html.contains("filterByLevel"));
        assert!(html.contains("toggleDetails"));
    }

    #[test]
    fn test_long_message_truncation() {
        let long_msg = "a".repeat(300);
        let entry = create_test_entry("1", "info", &long_msg, "src");
        let html = generate_dashboard_html(&[entry]);
        // Should contain truncated version
        assert!(html.contains("..."));
    }
}
