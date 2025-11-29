//! Summary generation for Playwright MCP snapshots.

use crate::constants::*;
use crate::regex::REF_REGEX;
use serde::Serialize;

/// Summary information extracted from a snapshot.
#[derive(Debug, Clone, Serialize)]
pub struct SnapshotSummary {
    /// Current page URL
    pub page_url: String,
    /// Current page title
    pub page_title: String,
    /// Number of open tabs
    pub tab_count: usize,
    /// Number of blank (about:blank) tabs
    pub blank_tab_count: usize,
    /// Number of errors/warnings in console
    pub error_count: usize,
    /// Total number of elements with refs
    pub element_count: usize,
}

/// Parse summary from snapshot text.
pub fn parse_summary(text: &str) -> SnapshotSummary {
    let mut page_url = String::new();
    let mut page_title = String::new();
    let mut error_count = 0;

    let element_count = REF_REGEX.find_iter(text).count();

    for line in text.lines() {
        if line.starts_with(FIELD_PAGE_URL) {
            page_url = line.trim_start_matches(FIELD_PAGE_URL).trim().to_string();
        } else if line.starts_with(FIELD_PAGE_TITLE) {
            page_title = line.trim_start_matches(FIELD_PAGE_TITLE).trim().to_string();
        } else if line.contains(MARKER_ERROR) || line.contains(MARKER_WARNING) {
            error_count += 1;
        }
    }

    // Tab counting from dedicated section
    let tab_section = extract_section(text, SECTION_TABS, SECTION_END);
    let tab_lines: Vec<&str> = tab_section.lines().filter(|l| l.starts_with("- ")).collect();
    let tab_count = tab_lines.len();
    let blank_tab_count = tab_lines
        .iter()
        .filter(|l| l.contains("(about:blank)"))
        .count();

    SnapshotSummary {
        page_url,
        page_title,
        tab_count,
        blank_tab_count,
        error_count,
        element_count,
    }
}

/// Extract a section from snapshot text.
pub fn extract_section(text: &str, start_marker: &str, end_marker: &str) -> String {
    let mut result = Vec::new();
    let mut in_section = false;

    for line in text.lines() {
        if line.contains(start_marker) {
            in_section = true;
            if !start_marker.starts_with("```") {
                result.push(line.to_string());
            }
            continue;
        }

        if in_section {
            if line.starts_with(end_marker) && line != start_marker {
                break;
            }
            result.push(line.to_string());
        }
    }

    result.join("\n")
}

/// Extract page state section.
pub fn extract_page_state(text: &str) -> String {
    let mut result = Vec::new();
    let mut in_state = false;

    for line in text.lines() {
        if line.contains(SECTION_PAGE_STATE) {
            in_state = true;
            continue;
        }

        if in_state {
            if line.starts_with(FIELD_PAGE_SNAPSHOT) {
                result.push(line.to_string());
                break;
            }
            result.push(line.to_string());
        }
    }

    result.join("\n")
}

/// Count blank tabs in snapshot text.
pub fn count_blank_tabs(text: &str) -> usize {
    let tab_section = extract_section(text, SECTION_TABS, SECTION_END);
    tab_section
        .lines()
        .filter(|l| l.starts_with("- ") && l.contains("(about:blank)"))
        .count()
}
