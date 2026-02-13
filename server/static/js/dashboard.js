// Global state
let eventSource = null;
let currentSortOrder = 'desc';
let allLogsData = [];

// LocalStorage keys for preferences
const PREFS_KEY = 'log-dashboard-prefs';

// Load preferences from localStorage
function loadPreferences() {
    try {
        const saved = localStorage.getItem(PREFS_KEY);
        if (saved) {
            const prefs = JSON.parse(saved);
            console.log('Loaded preferences:', prefs);
            return prefs;
        }
        console.log('No saved preferences found');
        return null;
    } catch (e) {
        console.error('Failed to load preferences:', e);
        return null;
    }
}

// Save preferences to localStorage
function savePreferences() {
    const prefs = {
        levels: Array.from(document.querySelectorAll('.level-filter:checked')).map(cb => cb.value),
        tags: Array.from(document.querySelectorAll('.tag-filter:checked')).map(cb => cb.value),
        source: document.getElementById('source-select')?.value || '',
        sortOrder: currentSortOrder,
        liveStream: document.getElementById('live-stream')?.checked ?? true,
        autoScroll: document.getElementById('autoscroll')?.checked ?? true,
        columnWidths: getColumnWidths(),
        hiddenColumns: Array.from(document.querySelectorAll('.column-toggle:not(:checked)')).map(cb => cb.value)
    };
    try {
        localStorage.setItem(PREFS_KEY, JSON.stringify(prefs));
        console.log('Saved preferences:', prefs);
    } catch (e) {
        console.error('Failed to save preferences:', e);
    }
}

// Get current column widths
function getColumnWidths() {
    const headers = document.querySelectorAll('#log-table thead th');
    const widths = {};
    headers.forEach((th, i) => {
        if (th.dataset.col) {
            widths[th.dataset.col] = th.style.width || th.offsetWidth + 'px';
        }
    });
    return widths;
}

// Apply saved preferences
function applyPreferences(prefs) {
    if (!prefs) return;

    // Apply level filters
    if (prefs.levels) {
        document.querySelectorAll('.level-filter').forEach(cb => {
            cb.checked = prefs.levels.includes(cb.value);
        });
    }

    // Apply tag filters (only for tags that exist)
    if (prefs.tags) {
        document.querySelectorAll('.tag-filter').forEach(cb => {
            cb.checked = prefs.tags.includes(cb.value);
        });
    }

    // Apply source filter
    if (prefs.source !== undefined) {
        const sourceSelect = document.getElementById('source-select');
        const sourceLabel = document.getElementById('source-label');
        if (sourceSelect) sourceSelect.value = prefs.source;
        if (sourceLabel) {
            sourceLabel.textContent = prefs.source || 'All Sources';
        }
    }

    // Apply sort order
    if (prefs.sortOrder) {
        currentSortOrder = prefs.sortOrder;
        const sortSelect = document.getElementById('sort-order');
        const sortLabel = document.getElementById('sort-label');
        if (sortSelect) sortSelect.value = prefs.sortOrder;
        if (sortLabel) {
            sortLabel.textContent = prefs.sortOrder === 'desc' ? 'Newest First' : 'Oldest First';
        }
    }

    // Apply live stream
    if (typeof prefs.liveStream === 'boolean') {
        const liveCheckbox = document.getElementById('live-stream');
        if (liveCheckbox) liveCheckbox.checked = prefs.liveStream;
    }

    // Apply auto scroll
    if (typeof prefs.autoScroll === 'boolean') {
        const autoScrollCheckbox = document.getElementById('autoscroll');
        if (autoScrollCheckbox) autoScrollCheckbox.checked = prefs.autoScroll;
    }

    // Apply column widths
    if (prefs.columnWidths) {
        applyColumnWidths(prefs.columnWidths);
    }

    // Apply hidden columns
    if (prefs.hiddenColumns) {
        document.querySelectorAll('.column-toggle').forEach(cb => {
            cb.checked = !prefs.hiddenColumns.includes(cb.value);
        });
        prefs.hiddenColumns.forEach(col => toggleColumnVisibility(col, false));
        updateColumnsCount();
    }
}

// Apply column widths from saved preferences
function applyColumnWidths(widths) {
    Object.keys(widths).forEach(col => {
        const th = document.querySelector(`#log-table thead th[data-col="${col}"]`);
        if (th) {
            th.style.width = widths[col];
        }
    });
}

// Toggle column visibility
function toggleColumnVisibility(column, visible) {
    document.querySelectorAll(`[data-column="${column}"]`).forEach(el => {
        el.classList.toggle('hidden', !visible);
    });
}

// Update columns count badge
function updateColumnsCount() {
    const count = document.querySelectorAll('.column-toggle:checked').length;
    const badge = document.getElementById('columns-count');
    if (badge) badge.textContent = count;
}

// Handle column toggle change
function filterByColumns() {
    document.querySelectorAll('.column-toggle').forEach(cb => {
        toggleColumnVisibility(cb.value, cb.checked);
    });
    updateColumnsCount();
    savePreferences();
}

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
                allLogsData.push(JSON.parse(decodeHtmlEntities(row.dataset.json)));
            } catch(e) {
                console.error('Failed to parse log data:', e);
            }
        }
    });
}

function decodeHtmlEntities(str) {
    const txt = document.createElement('textarea');
    txt.innerHTML = str;
    return txt.value;
}

// Filter functions
function filterByLevel() {
    updateLevelCount();
    applyAllFilters();
    savePreferences();
}

function filterBySource() {
    applyAllFilters();
    savePreferences();
}
const debouncedSearch = debounce(() => applyAllFilters(), 300);
function searchLogs() { debouncedSearch(); }

function updateLevelCount() {
    const count = document.querySelectorAll('.level-filter:checked').length;
    const badge = document.getElementById('level-count');
    if (badge) badge.textContent = count;
}

function filterByTags() {
    updateTagsCount();
    applyAllFilters();
    savePreferences();
}

function updateTagsCount() {
    const count = document.querySelectorAll('.tag-filter:checked').length;
    const badge = document.getElementById('tags-count');
    if (badge) badge.textContent = count;
}

function applyAllFilters() {
    const selectedLevels = Array.from(document.querySelectorAll('.level-filter:checked')).map(cb => cb.value);
    const selectedTags = Array.from(document.querySelectorAll('.tag-filter:checked')).map(cb => cb.value);
    const allTags = Array.from(document.querySelectorAll('.tag-filter')).map(cb => cb.value);
    const uncheckedTags = allTags.filter(t => !selectedTags.includes(t));
    const selectedSource = document.getElementById('source-select').value;
    const searchQuery = document.getElementById('search').value.toLowerCase();

    let visibleCount = 0;
    document.querySelectorAll('.log-row').forEach(row => {
        const level = row.dataset.level;
        const source = row.dataset.source;
        const message = (row.dataset.message || '').toLowerCase();
        const rowTags = (row.dataset.tags || '').split(',').filter(t => t.trim());

        let visible = selectedLevels.includes(level);
        if (visible && selectedSource && source !== selectedSource) visible = false;

        // Tag filter: Exclusive logic - hide if row has ANY unchecked tag
        // If all tags selected, show all. If row has no tags, show it.
        if (visible && uncheckedTags.length > 0 && rowTags.length > 0) {
            const hasUncheckedTag = rowTags.some(tag => uncheckedTags.includes(tag.trim()));
            if (hasUncheckedTag) visible = false;
        }

        if (visible && searchQuery && !message.includes(searchQuery) && !source.toLowerCase().includes(searchQuery)) visible = false;

        if (visible) visibleCount++;
        row.classList.toggle('hidden', !visible);

        const detailRow = document.getElementById(`detail-${row.dataset.id}`);
        if (detailRow && !visible) detailRow.classList.add('hidden');
    });

    updateNoResultsMessage(visibleCount);
}

function toggleDetails(logId, event) {
    if (event.target.closest('button')) return; // Don't toggle if clicking copy button
    const detailRow = document.getElementById(`detail-${logId}`);
    if (!detailRow) return;
    detailRow.classList.toggle('hidden');
}

function manualRefresh() { location.reload(); }

function clearAllLogs() {
    if (!confirm('Are you sure you want to clear all logs?')) return;
    fetch('/logs', { method: 'DELETE' })
        .then(response => {
            if (response.ok) location.reload();
            else alert('Failed to clear logs');
        })
        .catch(err => alert('Error: ' + err.message));
}

// Sorting
function sortTable() {
    const tbody = document.getElementById('logs-tbody');
    if (!tbody) return;
    const rows = Array.from(tbody.querySelectorAll('.log-row'));

    rows.sort((a, b) => {
        const tsA = parseInt(a.dataset.timestamp) || 0;
        const tsB = parseInt(b.dataset.timestamp) || 0;
        return currentSortOrder === 'desc' ? tsB - tsA : tsA - tsB;
    });

    rows.forEach(row => {
        const detailRow = document.getElementById(`detail-${row.dataset.id}`);
        tbody.appendChild(row);
        if (detailRow) tbody.appendChild(detailRow);
    });
}

// Export functions
function exportJSON() {
    collectLogData();
    const json = JSON.stringify(allLogsData, null, 2);
    copyToClipboard(json);
    closeAllDropdowns();
}

function exportTXT() {
    collectLogData();
    const lines = allLogsData.map(entry => {
        const ts = new Date(entry.timestamp).toISOString();
        return `[${ts}] [${entry.level.toUpperCase()}] [${entry.source}] ${entry.message}`;
    });
    copyToClipboard(lines.join('\n'));
    closeAllDropdowns();
}

function copyLogEntry(logId, event) {
    event.stopPropagation();
    const row = document.querySelector(`.log-row[data-id="${logId}"]`);
    if (row && row.dataset.json) {
        try {
            const json = decodeHtmlEntities(row.dataset.json);
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
    if (feedback) {
        feedback.classList.add('show');
        setTimeout(() => feedback.classList.remove('show'), 2000);
    }
}

// Dropdown management
function toggleDropdown(dropdownId) {
    const dropdown = document.getElementById(dropdownId);
    const isHidden = dropdown.classList.contains('hidden');
    closeAllDropdowns();
    if (isHidden) dropdown.classList.remove('hidden');
}

function closeAllDropdowns() {
    document.querySelectorAll('#level-dropdown, #export-dropdown, #tags-dropdown, #columns-dropdown, #source-dropdown, #sort-dropdown').forEach(d => d.classList.add('hidden'));
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

function getLevelColor(level) {
    const colors = {
        trace: { dot: 'bg-gray-400', text: 'text-gray-600' },
        debug: { dot: 'bg-gray-400', text: 'text-gray-600' },
        info: { dot: 'bg-emerald-500', text: 'text-emerald-600' },
        notice: { dot: 'bg-sky-500', text: 'text-sky-600' },
        warning: { dot: 'bg-amber-500', text: 'text-amber-600' },
        error: { dot: 'bg-red-500', text: 'text-red-600' },
        critical: { dot: 'bg-fuchsia-500', text: 'text-fuchsia-600' }
    };
    return colors[level.toLowerCase()] || colors.info;
}

// FNV-1a hash for deterministic tag colors (matches Rust implementation)
// Uses Math.imul for proper 32-bit multiplication semantics
function fnv1aHash(str) {
    let hash = 2166136261 >>> 0;
    for (let i = 0; i < str.length; i++) {
        hash ^= str.charCodeAt(i);
        hash = Math.imul(hash, 16777619);
    }
    return hash >>> 0;
}

// Generate a unique HSL color for a tag based on its hash
// Returns inline style string for background and text color
function getTagColorStyle(tag) {
    const hash = fnv1aHash(tag);
    // Use golden ratio for better hue distribution
    const hue = (hash * 137.508) % 360;
    // Keep saturation and lightness in pleasant ranges
    const saturation = 65 + (hash % 20); // 65-85%
    const lightness = 85 + (hash % 10); // 85-95% for background
    const textLightness = 25 + (hash % 15); // 25-40% for text

    return {
        bg: `hsl(${hue}, ${saturation}%, ${lightness}%)`,
        text: `hsl(${hue}, ${saturation}%, ${textLightness}%)`
    };
}

// Get inline style string for a tag
function getTagStyle(tag) {
    const colors = getTagColorStyle(tag);
    return `background-color: ${colors.bg}; color: ${colors.text};`;
}

// Format time for display (HH:MM:SS)
function formatTimeShort(isoString) {
    const date = new Date(isoString);
    const pad = (n) => String(n).padStart(2, '0');
    return `${pad(date.getUTCHours())}:${pad(date.getUTCMinutes())}:${pad(date.getUTCSeconds())}`;
}

// Get caller display (filename:line)
function getCallerDisplay(file, line) {
    if (!file && !line) return '-';
    const filename = file ? file.split('/').pop() : '';
    if (filename && line > 0) return `${filename}:${line}`;
    if (filename) return filename;
    return '-';
}

// Add new log entry (for SSE)
function prependLogEntry(entry) {
    const tbody = document.getElementById('logs-tbody');
    if (!tbody) return;

    const levelLower = entry.level.toLowerCase();
    const colors = getLevelColor(entry.level);
    const timeShort = formatTimeShort(entry.timestamp);
    const callerDisplay = getCallerDisplay(entry.file, entry.line);
    const hasCaller = entry.file || entry.line > 0;
    const entryJson = escapeHtml(JSON.stringify(entry));
    // Ensure tags is always an array, even if undefined/null in JSON
    const tags = Array.isArray(entry.tags) ? entry.tags : [];

    // Update source dropdown if new source
    updateSourceDropdown(entry.source);

    // Update tags dropdown if new tags
    if (tags.length > 0) {
        updateTagsDropdown(tags);
    }

    const tr = document.createElement('tr');
    tr.className = 'log-row hover:bg-gray-50 transition-colors cursor-pointer group animate-in';
    tr.dataset.level = levelLower;
    tr.dataset.source = entry.source;
    tr.dataset.message = entry.message;
    tr.dataset.id = entry.id;
    tr.dataset.timestamp = new Date(entry.timestamp).getTime();
    tr.dataset.file = entry.file || '';
    tr.dataset.line = entry.line || 0;
    tr.dataset.function = entry.function || '';
    tr.dataset.tags = tags.join(',');
    tr.dataset.json = entryJson;
    tr.onclick = (e) => toggleDetails(entry.id, e);

    // Render tags as pills with inline HSL colors (all values escaped for XSS protection)
    const tagsHtml = tags.length > 0
        ? `<div class="flex flex-wrap gap-1">${tags.map(tag =>
            `<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium" style="${getTagStyle(tag)}" title="${escapeHtml(tag)}">${escapeHtml(tag)}</span>`
          ).join('')}</div>`
        : '<span class="text-xs text-gray-400">-</span>';

    tr.innerHTML = `
        <td class="px-3 py-2 tags-cell-level" data-column="level">
            <span class="inline-flex items-center gap-1 text-xs font-medium ${colors.text}">
                <span class="w-2 h-2 rounded-full flex-shrink-0 ${colors.dot}"></span>
                ${entry.level.toUpperCase()}
            </span>
        </td>
        <td class="px-3 py-2 text-xs text-gray-500 font-mono whitespace-nowrap" data-column="time">${timeShort}</td>
        <td class="px-3 py-2" data-column="source">
            <span class="text-xs text-gray-600 font-medium truncate block" title="${escapeHtml(entry.source)}">${escapeHtml(entry.source)}</span>
        </td>
        <td class="px-3 py-2 tags-cell" data-column="tags">
            ${tagsHtml}
        </td>
        <td class="px-3 py-2" data-column="caller">
            ${hasCaller
                ? `<span class="text-xs text-gray-500 font-mono truncate block caller-cell" data-file="${escapeHtml(entry.file || '')}" data-line="${entry.line || 0}" title="${escapeHtml(entry.file || '')}:${entry.line || 0}">${escapeHtml(callerDisplay)}</span>`
                : '<span class="text-xs text-gray-400">-</span>'}
        </td>
        <td class="px-3 py-2" data-column="message">
            <div class="flex items-center gap-2">
                <span class="text-sm text-gray-800 font-mono truncate flex-1" title="${escapeHtml(entry.message)}">${escapeHtml(entry.message)}</span>
                <button class="flex-shrink-0 opacity-0 group-hover:opacity-100 p-1 text-gray-400 hover:text-gray-600 transition" onclick="copyLogEntry('${escapeHtml(entry.id)}', event)" title="Copy JSON">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
                    </svg>
                </button>
            </div>
        </td>
    `;

    // Apply current column visibility to new row
    document.querySelectorAll('.column-toggle').forEach(cb => {
        if (!cb.checked) {
            const cell = tr.querySelector(`[data-column="${cb.value}"]`);
            if (cell) cell.classList.add('hidden');
        }
    });

    if (currentSortOrder === 'desc') {
        tbody.insertBefore(tr, tbody.firstChild);
    } else {
        tbody.appendChild(tr);
    }

    // Remove animation class after animation completes
    tr.addEventListener('animationend', () => {
        tr.classList.remove('animate-in');
    }, { once: true });

    // Always create detail row for full message display
    const detailTr = document.createElement('tr');
    detailTr.className = 'detail-row hidden bg-gray-50';
    detailTr.id = `detail-${entry.id}`;

    let detailsHtml = '<td colspan="6" class="px-4 py-4"><div class="space-y-3">';
    detailsHtml += '<div><span class="text-gray-500 text-xs uppercase tracking-wide block mb-1">Full Message</span>';
    detailsHtml += `<pre class="text-gray-800 font-mono text-sm bg-white px-3 py-2 rounded border border-gray-200 whitespace-pre-wrap break-words">${escapeHtml(entry.message)}</pre></div>`;
    detailsHtml += '<div class="grid grid-cols-[100px_1fr] gap-y-2 gap-x-4 text-sm">';
    detailsHtml += `<span class="text-gray-500 text-xs uppercase tracking-wide">ID</span><code class="text-gray-800 font-mono text-xs bg-white px-2 py-1 rounded border border-gray-200">${escapeHtml(entry.id)}</code>`;
    detailsHtml += `<span class="text-gray-500 text-xs uppercase tracking-wide">Timestamp</span><code class="text-gray-800 font-mono text-xs bg-white px-2 py-1 rounded border border-gray-200">${formatTimestamp(entry.timestamp)}</code>`;
    if (entry.file) detailsHtml += `<span class="text-gray-500 text-xs uppercase tracking-wide">File</span><code class="text-gray-800 font-mono text-xs bg-white px-2 py-1 rounded border border-gray-200">${escapeHtml(entry.file)}</code>`;
    if (entry.function) detailsHtml += `<span class="text-gray-500 text-xs uppercase tracking-wide">Function</span><code class="text-gray-800 font-mono text-xs bg-white px-2 py-1 rounded border border-gray-200">${escapeHtml(entry.function)}</code>`;
    if (entry.line > 0) detailsHtml += `<span class="text-gray-500 text-xs uppercase tracking-wide">Line</span><code class="text-gray-800 font-mono text-xs bg-white px-2 py-1 rounded border border-gray-200">${entry.line}</code>`;
    if (tags.length > 0) {
        detailsHtml += '<span class="text-gray-500 text-xs uppercase tracking-wide">Tags</span>';
        detailsHtml += '<div class="flex flex-wrap gap-1">';
        tags.forEach(tag => {
            detailsHtml += `<span class="inline-flex items-center px-2 py-1 rounded text-xs font-medium" style="${getTagStyle(tag)}">${escapeHtml(tag)}</span>`;
        });
        detailsHtml += '</div>';
    }
    if (entry.metadata && Object.keys(entry.metadata).length > 0) {
        detailsHtml += '<span class="text-gray-500 text-xs uppercase tracking-wide">Metadata</span><pre class="text-gray-800 font-mono text-xs bg-white px-2 py-1 rounded border border-gray-200 overflow-x-auto">';
        for (const [key, value] of Object.entries(entry.metadata)) {
            detailsHtml += `${escapeHtml(key)}: ${escapeHtml(value)}\n`;
        }
        detailsHtml += '</pre>';
    }
    detailsHtml += '</div></div></td>';
    detailTr.innerHTML = detailsHtml;

    if (currentSortOrder === 'desc') {
        tbody.insertBefore(detailTr, tr.nextSibling);
    } else {
        tbody.appendChild(detailTr);
    }

    updateStatistics(entry);
    applyAllFilters();

    if (document.getElementById('autoscroll')?.checked) {
        scrollToTop();
    }
}

// Update source dropdown with new source if needed
function updateSourceDropdown(source) {
    const select = document.getElementById('source-select');
    const dropdown = document.getElementById('source-dropdown');
    if (!select || !dropdown) return;

    // Check if source already exists
    const exists = Array.from(select.options).some(opt => opt.value === source);
    if (exists) return;

    // Add new source option to hidden select in alphabetical order
    const newOption = document.createElement('option');
    newOption.value = source;
    newOption.textContent = source;

    // Find correct position (skip "All Sources" at index 0)
    let insertIndex = 1;
    for (let i = 1; i < select.options.length; i++) {
        if (select.options[i].value > source) {
            insertIndex = i;
            break;
        }
        insertIndex = i + 1;
    }
    select.insertBefore(newOption, select.options[insertIndex] || null);

    // Add new source button to styled dropdown
    const newBtn = document.createElement('button');
    newBtn.type = 'button';
    newBtn.className = 'source-option w-full text-left px-3 py-1.5 text-sm hover:bg-gray-50';
    newBtn.dataset.value = source;
    newBtn.textContent = source;
    newBtn.addEventListener('click', () => {
        document.getElementById('source-label').textContent = source;
        document.getElementById('source-select').value = source;
        closeAllDropdowns();
        filterBySource();
    });

    // Insert at correct position in dropdown (skip "All Sources" at index 0)
    const buttons = Array.from(dropdown.querySelectorAll('.source-option'));
    let insertBefore = null;
    for (let i = 1; i < buttons.length; i++) {
        if (buttons[i].dataset.value > source) {
            insertBefore = buttons[i];
            break;
        }
    }
    dropdown.insertBefore(newBtn, insertBefore);
}

// Update tags dropdown with new tags if needed
function updateTagsDropdown(tags) {
    const container = document.getElementById('tags-dropdown');
    if (!container || !tags || tags.length === 0) return;

    // Get existing tag values
    const existingTags = Array.from(container.querySelectorAll('.tag-filter')).map(cb => cb.value);

    // Add new tags
    tags.forEach(tag => {
        if (!existingTags.includes(tag)) {
            // Remove "No tags yet" message if present
            const noTagsMsg = container.querySelector('div.text-gray-400');
            if (noTagsMsg) noTagsMsg.remove();

            // Find footer (select all / clear all buttons)
            const footer = container.querySelector('.border-t');

            // Create new label with checkbox
            const label = document.createElement('label');
            label.className = 'flex items-center gap-2 px-3 py-1.5 hover:bg-gray-50 cursor-pointer';
            label.innerHTML = `
                <input type="checkbox" class="tag-filter rounded text-purple-600 focus:ring-purple-500" value="${escapeHtml(tag)}" checked>
                <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium" style="${getTagStyle(tag)}">
                    ${escapeHtml(tag)}
                </span>
            `;

            // Insert before footer or at end
            if (footer) {
                container.insertBefore(label, footer);
            } else {
                container.appendChild(label);
            }

            // Add event listener to new checkbox
            label.querySelector('.tag-filter').addEventListener('change', filterByTags);
        }
    });

    updateTagsCount();
    savePreferences();
}

// Update statistics
function updateStatistics(entry) {
    const totalEl = document.getElementById('total-count');
    if (totalEl) totalEl.textContent = parseInt(totalEl.textContent || 0) + 1;

    const levelCounts = document.getElementById('level-counts');
    if (levelCounts) {
        let pill = levelCounts.querySelector(`span[data-level="${entry.level.toLowerCase()}"]`);
        if (pill) {
            const countSpan = pill.querySelector('.count');
            if (countSpan) countSpan.textContent = parseInt(countSpan.textContent || 0) + 1;
        }
    }
}

// SSE connection
function initializeSSE() {
    const statusEl = document.getElementById('stream-status');
    if (eventSource) eventSource.close();

    if (statusEl) statusEl.className = 'w-2 h-2 rounded-full bg-yellow-400';

    eventSource = new EventSource('/stream');

    eventSource.addEventListener('log', function(event) {
        try {
            prependLogEntry(JSON.parse(event.data));
        } catch (e) {
            console.error('Failed to parse log entry:', e);
        }
    });

    eventSource.onopen = () => {
        if (statusEl) statusEl.className = 'w-2 h-2 rounded-full bg-green-500';
    };

    eventSource.onerror = () => {
        if (statusEl) statusEl.className = 'w-2 h-2 rounded-full bg-red-500';
        setTimeout(() => {
            if (document.getElementById('live-stream')?.checked) initializeSSE();
        }, 5000);
    };
}

function closeSSE() {
    if (eventSource) {
        eventSource.close();
        eventSource = null;
        const statusEl = document.getElementById('stream-status');
        if (statusEl) statusEl.className = 'w-2 h-2 rounded-full bg-gray-400';
    }
}

function setupLiveStreaming() {
    const checkbox = document.getElementById('live-stream');
    if (!checkbox) return;
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
            document.getElementById('search')?.focus();
        }
        if (event.key === 'Escape') {
            closeAllDropdowns();
        }
    });
}

function updateNoResultsMessage(visibleCount) {
    const msg = document.getElementById('no-results-message');
    const hasLogs = document.querySelectorAll('.log-row').length > 0;
    if (msg) {
        msg.classList.toggle('hidden', visibleCount > 0 || !hasLogs);
    }
}

// Prettify caller cells to show just filename
function prettifyCallerCells() {
    document.querySelectorAll('.caller-cell').forEach(cell => {
        const file = cell.dataset.file || '';
        const line = cell.dataset.line || '0';
        if (file) {
            const filename = file.split('/').pop();
            cell.textContent = line && line !== '0' ? `${filename}:${line}` : filename;
        }
    });
}

// Apply inline HSL colors to existing tag elements (server-rendered)
function applyTagColors() {
    const pills = document.querySelectorAll('.tag-pill');
    console.log(`Applying colors to ${pills.length} tag pills`);
    pills.forEach(pill => {
        const tag = pill.dataset.tag || pill.textContent.trim();
        if (tag) {
            const colors = getTagColorStyle(tag);
            pill.style.backgroundColor = colors.bg;
            pill.style.color = colors.text;
            pill.style.border = 'none';
        }
    });
}

// Column resize functionality
let resizingColumn = null;
let startX = 0;
let startWidth = 0;

function setupColumnResize() {
    const table = document.getElementById('log-table');
    if (!table) {
        console.log('Column resize: table #log-table not found');
        return;
    }

    const headers = table.querySelectorAll('thead th[data-col]');
    console.log(`Setting up resize for ${headers.length} columns`);

    headers.forEach(th => {
        // Skip last column (message) - it should fill remaining space
        if (th.dataset.col === 'message') return;

        // Create resize handle - visible gray line that turns purple on hover
        const handle = document.createElement('div');
        handle.className = 'resize-handle';
        handle.style.cssText = `
            position: absolute;
            right: -2px;
            top: 4px;
            bottom: 4px;
            width: 4px;
            cursor: col-resize;
            background: #e5e7eb;
            border-radius: 2px;
            z-index: 10;
            transition: background 0.15s;
        `;
        handle.addEventListener('mouseenter', () => handle.style.background = '#9333ea');
        handle.addEventListener('mouseleave', () => { if (!resizingColumn) handle.style.background = '#e5e7eb'; });

        // Position the th relatively for the handle
        th.style.position = 'relative';
        th.appendChild(handle);

        handle.addEventListener('mousedown', (e) => {
            e.preventDefault();
            e.stopPropagation();
            resizingColumn = th;
            startX = e.pageX;
            startWidth = th.offsetWidth;
            handle.style.background = '#9333ea';
            document.body.style.cursor = 'col-resize';
            document.body.style.userSelect = 'none';
        });
    });

    document.addEventListener('mousemove', (e) => {
        if (!resizingColumn) return;

        const diff = e.pageX - startX;
        const newWidth = Math.max(50, startWidth + diff);
        resizingColumn.style.width = newWidth + 'px';
    });

    document.addEventListener('mouseup', () => {
        if (resizingColumn) {
            const handle = resizingColumn.querySelector('.resize-handle');
            if (handle) handle.style.background = '#e5e7eb';
            resizingColumn = null;
            document.body.style.cursor = '';
            document.body.style.userSelect = '';
            savePreferences();
        }
    });
}

// Initialize on page load
document.addEventListener('DOMContentLoaded', function() {
    // Load saved preferences first
    const prefs = loadPreferences();

    // Apply tag colors to server-rendered tags
    applyTagColors();

    // Prettify caller cells
    prettifyCallerCells();

    // Set up column resizing
    setupColumnResize();

    // Apply saved preferences before setting up listeners
    applyPreferences(prefs);

    // Level dropdown
    const levelBtn = document.getElementById('level-dropdown-btn');
    if (levelBtn) {
        levelBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            toggleDropdown('level-dropdown');
        });
    }

    // Level checkboxes
    document.querySelectorAll('.level-filter').forEach(cb => {
        cb.addEventListener('change', filterByLevel);
    });

    // Select all / clear all levels
    document.getElementById('select-all-levels')?.addEventListener('click', () => {
        document.querySelectorAll('.level-filter').forEach(cb => cb.checked = true);
        filterByLevel();
    });
    document.getElementById('clear-all-levels')?.addEventListener('click', () => {
        document.querySelectorAll('.level-filter').forEach(cb => cb.checked = false);
        filterByLevel();
    });

    // Tags dropdown
    const tagsBtn = document.getElementById('tags-dropdown-btn');
    if (tagsBtn) {
        tagsBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            toggleDropdown('tags-dropdown');
        });
    }

    // Tags checkboxes
    document.querySelectorAll('.tag-filter').forEach(cb => {
        cb.addEventListener('change', filterByTags);
    });

    // Select all / clear all tags
    document.getElementById('select-all-tags')?.addEventListener('click', () => {
        document.querySelectorAll('.tag-filter').forEach(cb => cb.checked = true);
        filterByTags();
    });
    document.getElementById('clear-all-tags')?.addEventListener('click', () => {
        document.querySelectorAll('.tag-filter').forEach(cb => cb.checked = false);
        filterByTags();
    });

    // Columns dropdown
    const columnsBtn = document.getElementById('columns-dropdown-btn');
    if (columnsBtn) {
        columnsBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            toggleDropdown('columns-dropdown');
        });
    }

    // Column toggle checkboxes
    document.querySelectorAll('.column-toggle').forEach(cb => {
        cb.addEventListener('change', filterByColumns);
    });

    // Show all columns button
    document.getElementById('show-all-columns')?.addEventListener('click', () => {
        document.querySelectorAll('.column-toggle').forEach(cb => cb.checked = true);
        filterByColumns();
    });

    // Source dropdown
    const sourceBtn = document.getElementById('source-dropdown-btn');
    if (sourceBtn) {
        sourceBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            toggleDropdown('source-dropdown');
        });
    }

    // Source options
    document.querySelectorAll('.source-option').forEach(btn => {
        btn.addEventListener('click', (e) => {
            const value = btn.dataset.value;
            const label = btn.textContent.trim();
            document.getElementById('source-label').textContent = label;
            document.getElementById('source-select').value = value;
            closeAllDropdowns();
            filterBySource();
        });
    });

    // Search
    document.getElementById('search')?.addEventListener('input', searchLogs);

    // Sort dropdown
    const sortBtn = document.getElementById('sort-dropdown-btn');
    if (sortBtn) {
        sortBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            toggleDropdown('sort-dropdown');
        });
    }

    // Sort options
    document.querySelectorAll('.sort-option').forEach(btn => {
        btn.addEventListener('click', (e) => {
            const value = btn.dataset.value;
            const label = btn.textContent.trim();
            document.getElementById('sort-label').textContent = label;
            document.getElementById('sort-order').value = value;
            currentSortOrder = value;
            closeAllDropdowns();
            sortTable();
            savePreferences();
        });
    });

    // Live stream toggle with preference saving
    document.getElementById('live-stream')?.addEventListener('change', savePreferences);

    // Auto-scroll toggle with preference saving
    document.getElementById('autoscroll')?.addEventListener('change', savePreferences);

    // Export dropdown
    const exportBtn = document.getElementById('export-dropdown-btn');
    if (exportBtn) {
        exportBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            toggleDropdown('export-dropdown');
        });
    }
    document.getElementById('export-json')?.addEventListener('click', exportJSON);
    document.getElementById('export-txt')?.addEventListener('click', exportTXT);

    // Utility buttons
    document.getElementById('refresh')?.addEventListener('click', manualRefresh);
    document.getElementById('clear-logs')?.addEventListener('click', clearAllLogs);

    // Close dropdowns on outside click
    document.addEventListener('click', (e) => {
        if (!e.target.closest('#level-dropdown-container') &&
            !e.target.closest('#export-dropdown-container') &&
            !e.target.closest('#tags-dropdown-container') &&
            !e.target.closest('#columns-dropdown-container') &&
            !e.target.closest('#source-dropdown-container') &&
            !e.target.closest('#sort-dropdown-container')) {
            closeAllDropdowns();
        }
    });

    setupLiveStreaming();
    setupKeyboardShortcuts();
    updateLevelCount();
    updateTagsCount();
    collectLogData();

    // Apply filters based on loaded preferences
    if (prefs) {
        applyAllFilters();
    }
});
