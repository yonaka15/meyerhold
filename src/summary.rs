//! Summary generation for Playwright MCP snapshots.

use crate::constants::*;
use crate::elements::{extract_elements, Element, ListType};
use crate::regex::REF_REGEX;
use serde::Serialize;

/// Default character limit for text preview.
pub const DEFAULT_TEXT_CHAR_LIMIT: usize = 20_000;

/// Default character limit for element labels.
pub const DEFAULT_LABEL_CHAR_LIMIT: usize = 50;

/// A simplified element representation for summary output.
#[derive(Debug, Clone, Serialize)]
pub struct SummaryElement {
    /// Element reference ID (e.g., "e407")
    pub ref_id: String,
    /// Element label/text content
    pub label: String,
}

/// Content item representing either a heading or text in page order.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ContentItem {
    /// A heading element
    #[serde(rename = "heading")]
    Heading { label: String },
    /// A text element
    #[serde(rename = "text")]
    Text { label: String },
}

impl From<Element> for SummaryElement {
    fn from(elem: Element) -> Self {
        Self {
            ref_id: elem.ref_id,
            label: truncate_label(&elem.label, DEFAULT_LABEL_CHAR_LIMIT),
        }
    }
}

/// Truncate label at character boundary (UTF-8 safe).
fn truncate_label(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        return s.to_string();
    }

    let truncate_at = max_chars.saturating_sub(3);
    let end_pos = s
        .char_indices()
        .nth(truncate_at)
        .map(|(pos, _)| pos)
        .unwrap_or(s.len());

    format!("{}...", &s[..end_pos])
}

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
    /// Buttons on the page
    pub buttons: Vec<SummaryElement>,
    /// Links on the page
    pub links: Vec<SummaryElement>,
    /// Input elements (textbox, checkbox, etc.)
    pub inputs: Vec<SummaryElement>,
    /// Page content (headings and text in order)
    pub content: Vec<ContentItem>,
    /// Whether content was truncated due to character limit
    pub content_truncated: bool,
}

/// Parse summary from snapshot text.
pub fn parse_summary(text: &str) -> SnapshotSummary {
    parse_summary_with_limit(text, DEFAULT_TEXT_CHAR_LIMIT)
}

/// Parse summary from snapshot text with custom character limit for texts.
pub fn parse_summary_with_limit(text: &str, char_limit: usize) -> SnapshotSummary {
    let mut page_url = String::new();
    let mut page_title = String::new();
    let mut error_count = 0;
    let mut content = Vec::new();
    let mut total_chars = 0;
    let mut content_truncated = false;

    let element_count = REF_REGEX.find_iter(text).count();

    for line in text.lines() {
        if line.starts_with(FIELD_PAGE_URL) {
            page_url = line.trim_start_matches(FIELD_PAGE_URL).trim().to_string();
        } else if line.starts_with(FIELD_PAGE_TITLE) {
            page_title = line.trim_start_matches(FIELD_PAGE_TITLE).trim().to_string();
        } else if line.contains(MARKER_ERROR) || line.contains(MARKER_WARNING) {
            error_count += 1;
        }

        let trimmed = line.trim().trim_start_matches("- ");

        // Extract heading content
        if trimmed.starts_with("heading") {
            if let Some(label) = extract_quoted_content(trimmed) {
                if !label.is_empty() {
                    let label_len = label.chars().count();
                    if total_chars + label_len <= char_limit {
                        total_chars += label_len;
                        content.push(ContentItem::Heading { label });
                    } else {
                        content_truncated = true;
                    }
                }
            }
        }
        // Extract text content
        else if trimmed.starts_with("text:") {
            let label = trimmed
                .trim_start_matches("text:")
                .trim()
                .trim_matches('"')
                .to_string();
            if !label.is_empty() {
                let label_len = label.chars().count();
                if total_chars + label_len <= char_limit {
                    total_chars += label_len;
                    content.push(ContentItem::Text { label });
                } else {
                    content_truncated = true;
                }
            }
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

    // Extract interactive elements
    let buttons: Vec<SummaryElement> = extract_elements(text, ListType::Buttons)
        .into_iter()
        .map(SummaryElement::from)
        .collect();
    let links: Vec<SummaryElement> = extract_elements(text, ListType::Links)
        .into_iter()
        .map(SummaryElement::from)
        .collect();
    let inputs: Vec<SummaryElement> = extract_elements(text, ListType::Inputs)
        .into_iter()
        .map(SummaryElement::from)
        .collect();

    SnapshotSummary {
        page_url,
        page_title,
        tab_count,
        blank_tab_count,
        error_count,
        element_count,
        buttons,
        links,
        inputs,
        content,
        content_truncated,
    }
}

/// Extract quoted content from a line (e.g., `heading "Title"` -> `Title`).
fn extract_quoted_content(line: &str) -> Option<String> {
    // Find content between quotes
    if let Some(start) = line.find('"') {
        if let Some(end) = line.rfind('"') {
            if start < end {
                return Some(line[start + 1..end].to_string());
            }
        }
    }
    None
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
