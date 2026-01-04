//! Tree navigation for Playwright MCP snapshots.

use crate::constants::{SECTION_TREE_END, SECTION_TREE_START};
use crate::summary::{extract_section, ContentItem, DEFAULT_LABEL_CHAR_LIMIT};
use crate::utils::{
    calculate_depth, extract_quoted_content, set_item_depth, truncate_label, AncestorTracker,
};
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

    // Third pass: extract content below target ref with visible ancestor tracking
    let mut content = Vec::new();
    let mut in_subtree = false;
    // Track visible ancestors with O(1) depth calculation
    let mut tracker = AncestorTracker::new();

    for (idx, line) in lines.iter().enumerate() {
        let dom_depth = calculate_depth(line);

        if idx == target_idx {
            in_subtree = true;
            // Target item itself has visible_depth = 0
            if let Some(mut item) = extract_content_item_raw(line) {
                set_item_depth(&mut item, 0);
                content.push(item);
                tracker.push(dom_depth, true);
            } else {
                tracker.push(dom_depth, false);
            }
            continue;
        }

        if in_subtree {
            // Check if we've exited the subtree
            if dom_depth <= target_indent / 2 && line.trim().starts_with('-') {
                break;
            }

            // Pop ancestors that are not parents
            tracker.pop_non_ancestors(dom_depth);

            // Get visible depth (O(1))
            let visible_depth = tracker.visible_depth();

            // Extract content items
            if let Some(mut item) = extract_content_item_raw(line) {
                set_item_depth(&mut item, visible_depth);
                content.push(item);
                tracker.push(dom_depth, true);
            } else {
                tracker.push(dom_depth, false);
            }
        }
    }

    Some(ViewResult { path, content })
}


/// Extract a ContentItem from a tree line (with depth=0, to be set later).
fn extract_content_item_raw(line: &str) -> Option<ContentItem> {
    let trimmed = line.trim().trim_start_matches("- ");

    // Extract ref if present
    let ref_id = REF_EXTRACT_REGEX
        .captures(line)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        .unwrap_or_default();

    // Heading
    if trimmed.starts_with("heading") {
        let label = extract_quoted_content(trimmed).unwrap_or_default();
        if !label.is_empty() {
            return Some(ContentItem::Heading {
                ref_id,
                label: truncate_label(&label, DEFAULT_LABEL_CHAR_LIMIT),
                depth: 0,
            });
        }
    }
    // Button
    else if trimmed.starts_with("button") {
        let label = extract_quoted_content(trimmed).unwrap_or_default();
        return Some(ContentItem::Button {
            ref_id,
            label: truncate_label(&label, DEFAULT_LABEL_CHAR_LIMIT),
            depth: 0,
        });
    }
    // Link
    else if trimmed.starts_with("link") {
        let label = extract_quoted_content(trimmed).unwrap_or_default();
        return Some(ContentItem::Link {
            ref_id,
            label: truncate_label(&label, DEFAULT_LABEL_CHAR_LIMIT),
            depth: 0,
        });
    }
    // Input types
    else if trimmed.starts_with("textbox")
        || trimmed.starts_with("searchbox")
        || trimmed.starts_with("combobox")
        || trimmed.starts_with("checkbox")
        || trimmed.starts_with("radio")
    {
        let label = extract_quoted_content(trimmed).unwrap_or_default();
        return Some(ContentItem::Input {
            ref_id,
            label: truncate_label(&label, DEFAULT_LABEL_CHAR_LIMIT),
            depth: 0,
        });
    }
    // Text element
    else if trimmed.starts_with("text:") {
        let label = trimmed.trim_start_matches("text:").trim().trim_matches('"');
        // Check if label has meaningful content (at least one alphanumeric char)
        let has_content = label.chars().any(|c| c.is_alphanumeric());
        // Skip only if BOTH blank AND no ref
        if has_content || !ref_id.is_empty() {
            return Some(ContentItem::Text {
                ref_id,
                label: label.to_string(),
                depth: 0,
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
                        depth: 0,
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
                    depth: 0,
                });
            }
        }
    }

    None
}
