//! Summary generation for Playwright MCP snapshots.

use crate::constants::*;
use crate::regex::REF_REGEX;
use crate::utils::{calculate_depth, extract_quoted_content, truncate_label, AncestorTracker};
use serde::Serialize;

/// Default character limit for text preview.
pub const DEFAULT_TEXT_CHAR_LIMIT: usize = 20_000;

/// Default character limit for element labels.
pub const DEFAULT_LABEL_CHAR_LIMIT: usize = 50;

/// Content item representing any element in page/DOM order.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ContentItem {
    /// A heading element
    #[serde(rename = "heading")]
    Heading {
        ref_id: String,
        label: String,
        depth: usize,
    },
    /// A text element
    #[serde(rename = "text")]
    Text {
        ref_id: String,
        label: String,
        depth: usize,
    },
    /// A button element
    #[serde(rename = "button")]
    Button {
        ref_id: String,
        label: String,
        depth: usize,
    },
    /// A link element
    #[serde(rename = "link")]
    Link {
        ref_id: String,
        label: String,
        depth: usize,
    },
    /// An input element (textbox, checkbox, etc.)
    #[serde(rename = "input")]
    Input {
        ref_id: String,
        label: String,
        depth: usize,
    },
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
    /// Page content (all elements in DOM order)
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
    use crate::regex::REF_REGEX as LINE_REF_REGEX;

    let mut page_url = String::new();
    let mut page_title = String::new();
    let mut error_count = 0;
    let mut content = Vec::new();
    let mut total_chars = 0;
    let mut content_truncated = false;

    let element_count = REF_REGEX.find_iter(text).count();

    // Input element types
    let input_types = [
        ELEM_TEXTBOX,
        ELEM_CHECKBOX,
        ELEM_RADIO,
        ELEM_COMBOBOX,
        ELEM_SEARCHBOX,
    ];

    // Track visible ancestors with O(1) depth calculation
    let mut tracker = AncestorTracker::new();

    for line in text.lines() {
        if line.starts_with(FIELD_PAGE_URL) {
            page_url = line.trim_start_matches(FIELD_PAGE_URL).trim().to_string();
        } else if line.starts_with(FIELD_PAGE_TITLE) {
            page_title = line.trim_start_matches(FIELD_PAGE_TITLE).trim().to_string();
        } else if line.contains(MARKER_ERROR) || line.contains(MARKER_WARNING) {
            error_count += 1;
        }

        let trimmed = line.trim().trim_start_matches("- ");
        let dom_depth = calculate_depth(line);

        // Pop ancestors that are not parents (depth >= current)
        tracker.pop_non_ancestors(dom_depth);

        // Get visible depth (O(1))
        let visible_depth = tracker.visible_depth();

        // Track if this line is a content item
        let mut is_content_item = false;

        // Extract heading content
        if trimmed.starts_with(ELEM_HEADING) {
            if let Some(label) = extract_quoted_content(trimmed) {
                if !label.is_empty() {
                    let ref_id = LINE_REF_REGEX
                        .captures(line)
                        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
                        .unwrap_or_default();
                    let label_len = label.chars().count();
                    if total_chars + label_len <= char_limit {
                        total_chars += label_len;
                        content.push(ContentItem::Heading {
                            ref_id,
                            label,
                            depth: visible_depth,
                        });
                        is_content_item = true;
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
            let ref_id = LINE_REF_REGEX
                .captures(line)
                .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
                .unwrap_or_default();
            // Check if label has meaningful content (at least one alphanumeric char)
            let has_content = label.chars().any(|c| c.is_alphanumeric());
            // Skip only if BOTH blank AND no ref
            if has_content || !ref_id.is_empty() {
                let label_len = label.chars().count();
                if total_chars + label_len <= char_limit {
                    total_chars += label_len;
                    content.push(ContentItem::Text {
                        ref_id,
                        label,
                        depth: visible_depth,
                    });
                    is_content_item = true;
                } else {
                    content_truncated = true;
                }
            }
        }
        // Extract generic with text content (e.g., "generic [ref=e14]: ⌥")
        else if trimmed.starts_with("generic") && trimmed.contains(':') {
            if let Some(colon_pos) = trimmed.find(':') {
                let after_colon = trimmed[colon_pos + 1..].trim();
                // Only extract if there's actual text content (not just whitespace or children)
                if !after_colon.is_empty() && !after_colon.starts_with('-') {
                    let label = after_colon.trim_matches('"').to_string();
                    if !label.is_empty() {
                        let ref_id = LINE_REF_REGEX
                            .captures(line)
                            .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
                            .unwrap_or_default();
                        let label_len = label.chars().count();
                        if total_chars + label_len <= char_limit {
                            total_chars += label_len;
                            content.push(ContentItem::Text {
                                ref_id,
                                label,
                                depth: visible_depth,
                            });
                            is_content_item = true;
                        } else {
                            content_truncated = true;
                        }
                    }
                }
            }
        }
        // Extract paragraph with inline text content
        else if trimmed.starts_with("paragraph") && trimmed.contains(':') {
            if let Some(colon_pos) = trimmed.find(':') {
                let after_colon = trimmed[colon_pos + 1..].trim();
                if !after_colon.is_empty() && !after_colon.starts_with('-') {
                    let label = after_colon.to_string();
                    if !label.is_empty() {
                        let ref_id = LINE_REF_REGEX
                            .captures(line)
                            .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
                            .unwrap_or_default();
                        let label_len = label.chars().count();
                        if total_chars + label_len <= char_limit {
                            total_chars += label_len;
                            content.push(ContentItem::Text {
                                ref_id,
                                label,
                                depth: visible_depth,
                            });
                            is_content_item = true;
                        } else {
                            content_truncated = true;
                        }
                    }
                }
            }
        }
        // Extract button
        else if trimmed.starts_with(ELEM_BUTTON) {
            if let Some(caps) = LINE_REF_REGEX.captures(line) {
                let ref_id = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                let label = extract_quoted_content(trimmed).unwrap_or_default();
                let label = truncate_label(&label, DEFAULT_LABEL_CHAR_LIMIT);
                let label_len = label.chars().count();
                if total_chars + label_len <= char_limit {
                    total_chars += label_len;
                    content.push(ContentItem::Button {
                        ref_id,
                        label,
                        depth: visible_depth,
                    });
                    is_content_item = true;
                } else {
                    content_truncated = true;
                }
            }
        }
        // Extract link
        else if trimmed.starts_with(ELEM_LINK) {
            if let Some(caps) = LINE_REF_REGEX.captures(line) {
                let ref_id = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                let label = extract_quoted_content(trimmed).unwrap_or_default();
                let label = truncate_label(&label, DEFAULT_LABEL_CHAR_LIMIT);
                let label_len = label.chars().count();
                if total_chars + label_len <= char_limit {
                    total_chars += label_len;
                    content.push(ContentItem::Link {
                        ref_id,
                        label,
                        depth: visible_depth,
                    });
                    is_content_item = true;
                } else {
                    content_truncated = true;
                }
            }
        }
        // Extract input elements
        else if input_types.iter().any(|&t| trimmed.starts_with(t)) {
            if let Some(caps) = LINE_REF_REGEX.captures(line) {
                let ref_id = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                let label = extract_quoted_content(trimmed).unwrap_or_default();
                let label = truncate_label(&label, DEFAULT_LABEL_CHAR_LIMIT);
                let label_len = label.chars().count();
                if total_chars + label_len <= char_limit {
                    total_chars += label_len;
                    content.push(ContentItem::Input {
                        ref_id,
                        label,
                        depth: visible_depth,
                    });
                    is_content_item = true;
                } else {
                    content_truncated = true;
                }
            }
        }

        // Push current line to ancestor tracker
        tracker.push(dom_depth, is_content_item);
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
        content,
        content_truncated,
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
