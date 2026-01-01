//! Tree navigation for Playwright MCP snapshots.

use crate::constants::{SECTION_TREE_END, SECTION_TREE_START};
use crate::summary::{extract_section, ContentItem, DEFAULT_LABEL_CHAR_LIMIT};
use regex::Regex;
use std::sync::LazyLock;

/// Regex to extract ref from element.
static REF_EXTRACT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[ref=([a-zA-Z0-9-]+)\]").unwrap());

/// Navigate and extract tree structure from snapshot.
pub fn get_tree(text: &str, depth: usize, from_ref: Option<&str>) -> String {
    let tree_text = extract_section(text, SECTION_TREE_START, SECTION_TREE_END);
    let mut output_lines = Vec::new();
    let mut found_from = from_ref.is_none();
    let mut base_indent = 0;

    for line in tree_text.lines() {
        let indent = line.len() - line.trim_start().len();
        let line_depth = indent / 2;

        if let Some(ref_id) = from_ref {
            if !found_from {
                if line.contains(&format!("[ref={}]", ref_id)) {
                    found_from = true;
                    base_indent = indent;
                } else {
                    continue;
                }
            } else {
                let is_new_element = line.trim().starts_with('-') || line.contains("[ref=");
                if indent <= base_indent && is_new_element && !output_lines.is_empty() {
                    break;
                }
            }
        }

        let effective_depth = if from_ref.is_some() {
            (indent.saturating_sub(base_indent)) / 2
        } else {
            line_depth
        };

        if effective_depth < depth {
            output_lines.push(line.to_string());
        }
    }

    output_lines.join("\n")
}

/// View result containing path to ref and content below it.
#[derive(Debug)]
pub struct ViewResult {
    /// The path from root to the target ref (hierarchy)
    pub path: Vec<String>,
    /// Flat content items below the target ref
    pub content: Vec<ContentItem>,
}

/// View a specific ref: show path to it and flat content below.
///
/// # Arguments
///
/// * `text` - The snapshot text
/// * `target_ref` - The ref ID to view (e.g., "e407")
///
/// # Returns
///
/// `Some(ViewResult)` if ref found, `None` otherwise.
pub fn view_ref(text: &str, target_ref: &str) -> Option<ViewResult> {
    let tree_text = extract_section(text, SECTION_TREE_START, SECTION_TREE_END);
    let target_pattern = format!("[ref={}]", target_ref);

    // First pass: find the target ref and its indent level
    let mut target_line_idx = None;
    let mut target_indent = 0;
    let lines: Vec<&str> = tree_text.lines().collect();

    for (idx, line) in lines.iter().enumerate() {
        if line.contains(&target_pattern) {
            target_line_idx = Some(idx);
            target_indent = line.len() - line.trim_start().len();
            break;
        }
    }

    let target_idx = target_line_idx?;

    // Second pass: build path to target (trace ancestors)
    let mut path: Vec<String> = Vec::new();
    let mut current_indent = target_indent;

    // Walk backwards to find ancestors
    for idx in (0..=target_idx).rev() {
        let line = lines[idx];
        let line_indent = line.len() - line.trim_start().len();

        // Include lines that are at lower indent levels (ancestors)
        if line_indent < current_indent || idx == target_idx {
            // Only include element lines (start with - or contain ref)
            let trimmed = line.trim();
            if trimmed.starts_with('-') || line.contains("[ref=") {
                path.push(line.to_string());
                current_indent = line_indent;
            }
        }
    }

    // Reverse to get root-to-target order
    path.reverse();

    // Third pass: extract flat content below target ref
    let mut content = Vec::new();
    let mut in_subtree = false;

    for (idx, line) in lines.iter().enumerate() {
        if idx == target_idx {
            in_subtree = true;
            // Also extract content from the target line itself
            if let Some(item) = extract_content_item(line) {
                content.push(item);
            }
            continue;
        }

        if in_subtree {
            let indent = line.len() - line.trim_start().len();

            // Check if we've exited the subtree
            if indent <= target_indent && line.trim().starts_with('-') {
                break;
            }

            // Extract content items
            if let Some(item) = extract_content_item(line) {
                content.push(item);
            }
        }
    }

    Some(ViewResult { path, content })
}

/// Extract a ContentItem from a tree line if it's a relevant element.
fn extract_content_item(line: &str) -> Option<ContentItem> {
    let trimmed = line.trim().trim_start_matches("- ");

    // Extract ref if present
    let ref_id = REF_EXTRACT_REGEX
        .captures(line)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        .unwrap_or_default();

    // Heading
    if trimmed.starts_with("heading") {
        let label = extract_quoted_label(trimmed);
        if !label.is_empty() {
            return Some(ContentItem::Heading {
                ref_id,
                label: truncate_label(&label, DEFAULT_LABEL_CHAR_LIMIT),
            });
        }
    }
    // Button
    else if trimmed.starts_with("button") {
        let label = extract_quoted_label(trimmed);
        return Some(ContentItem::Button {
            ref_id,
            label: truncate_label(&label, DEFAULT_LABEL_CHAR_LIMIT),
        });
    }
    // Link
    else if trimmed.starts_with("link") {
        let label = extract_quoted_label(trimmed);
        return Some(ContentItem::Link {
            ref_id,
            label: truncate_label(&label, DEFAULT_LABEL_CHAR_LIMIT),
        });
    }
    // Input types
    else if trimmed.starts_with("textbox")
        || trimmed.starts_with("searchbox")
        || trimmed.starts_with("combobox")
        || trimmed.starts_with("checkbox")
        || trimmed.starts_with("radio")
    {
        let label = extract_quoted_label(trimmed);
        return Some(ContentItem::Input {
            ref_id,
            label: truncate_label(&label, DEFAULT_LABEL_CHAR_LIMIT),
        });
    }
    // Text element
    else if trimmed.starts_with("text:") {
        let label = trimmed.trim_start_matches("text:").trim().trim_matches('"');
        if !label.is_empty() {
            return Some(ContentItem::Text {
                ref_id,
                label: label.to_string(),
            });
        }
    }
    // Generic with text content
    else if trimmed.starts_with("generic") && trimmed.contains(':') {
        if let Some(colon_pos) = trimmed.find(':') {
            let after_colon = trimmed[colon_pos + 1..].trim();
            if !after_colon.is_empty() && !after_colon.starts_with('-') {
                let label = after_colon.trim_matches('"');
                if !label.is_empty() {
                    return Some(ContentItem::Text {
                        ref_id,
                        label: label.to_string(),
                    });
                }
            }
        }
    }
    // Paragraph with inline text
    else if trimmed.starts_with("paragraph") && trimmed.contains(':') {
        if let Some(colon_pos) = trimmed.find(':') {
            let after_colon = trimmed[colon_pos + 1..].trim();
            if !after_colon.is_empty() && !after_colon.starts_with('-') {
                return Some(ContentItem::Text {
                    ref_id,
                    label: after_colon.to_string(),
                });
            }
        }
    }

    None
}

/// Extract quoted label from element line.
fn extract_quoted_label(content: &str) -> String {
    if let Some(start) = content.find('"') {
        if let Some(end) = content.rfind('"') {
            if start < end {
                return content[start + 1..end].to_string();
            }
        }
    }
    String::new()
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
