//! Tree navigation for Playwright MCP snapshots.

use crate::constants::{SECTION_TREE_END, SECTION_TREE_START};
use crate::summary::{extract_section, ContentItem, DEFAULT_LABEL_CHAR_LIMIT};
use regex::Regex;
use std::sync::LazyLock;

/// Regex to match generic elements without content (container only).
static GENERIC_CONTAINER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^-\s*generic(\s+\[.*\])?\s*:?\s*$").unwrap());

/// Regex to match generic elements with text content.
static GENERIC_WITH_TEXT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^-\s*generic(\s+\[.*\])?:\s*(.+)$").unwrap());

/// Regex to extract ref from element.
static REF_EXTRACT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[ref=([a-zA-Z0-9-]+)\]").unwrap());

/// Get compact tree with semantic element symbols and 2-space indentation.
pub fn get_compact_tree(text: &str) -> String {
    let tree_text = extract_section(text, SECTION_TREE_START, SECTION_TREE_END);
    let mut output_lines = Vec::new();

    for line in tree_text.lines() {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Skip generic container elements (no content, just nesting)
        if trimmed.starts_with("- generic") && GENERIC_CONTAINER_REGEX.is_match(trimmed) {
            continue;
        }

        // Calculate indent level
        let indent = line.len() - line.trim_start().len();
        let depth = indent / 2;

        // Build depth prefix with 2-space indentation
        let prefix = "  ".repeat(depth);

        // Remove the leading "- " and whitespace, also handle YAML quoted lines
        let content = trimmed
            .trim_start_matches("- ")
            .trim_start_matches('\'')
            .trim_end_matches("':")
            .trim_end_matches(':');

        // Convert to semantic format
        let formatted = format_semantic(content);

        output_lines.push(format!("{}{}", prefix, formatted));
    }

    output_lines.join("\n")
}

/// Convert element to semantic format with type symbols.
fn format_semantic(content: &str) -> String {
    // Extract ref if present
    let ref_id = REF_EXTRACT_REGEX
        .captures(content)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()));

    // Check for generic with text content -> convert to [txt]
    if content.starts_with("generic") {
        if let Some(caps) = GENERIC_WITH_TEXT_REGEX.captures(&format!("- {}", content)) {
            if let Some(text) = caps.get(2) {
                let text_content = text.as_str().trim();
                if let Some(ref_id) = ref_id {
                    return format!("[txt] {} [{}]", text_content, ref_id);
                } else {
                    return format!("[txt] {}", text_content);
                }
            }
        }
    }

    // Element type mapping
    let (symbol, label) = if content.starts_with("button ") || content.starts_with("button\"") {
        ("[btn]", extract_label(content, "button"))
    } else if content.starts_with("link ") || content.starts_with("link\"") {
        ("[lnk]", extract_label(content, "link"))
    } else if content.starts_with("textbox ") || content.starts_with("textbox\"") {
        ("[inp]", extract_label(content, "textbox"))
    } else if content.starts_with("searchbox ") || content.starts_with("searchbox\"") {
        ("[inp]", extract_label(content, "searchbox"))
    } else if content.starts_with("combobox ") || content.starts_with("combobox\"") {
        ("[inp]", extract_label(content, "combobox"))
    } else if content.starts_with("checkbox ") || content.starts_with("checkbox\"") {
        ("[chk]", extract_label(content, "checkbox"))
    } else if content.starts_with("radio ") || content.starts_with("radio\"") {
        ("[rad]", extract_label(content, "radio"))
    } else if content.starts_with("img ") || content == "img" {
        ("[img]", extract_label(content, "img"))
    } else if content.starts_with("heading ") || content.starts_with("heading\"") {
        let level = if content.contains("[level=1]") {
            "[h1]"
        } else if content.contains("[level=2]") {
            "[h2]"
        } else if content.contains("[level=3]") {
            "[h3]"
        } else if content.contains("[level=4]") {
            "[h4]"
        } else {
            "[h]"
        };
        (level, extract_label(content, "heading"))
    } else if content.starts_with("paragraph ") || content.starts_with("paragraph:") {
        ("[p]", extract_paragraph_content(content))
    } else if content.starts_with("text:") {
        ("[txt]", content.trim_start_matches("text:").trim().to_string())
    } else if content.starts_with("strong ") || content.starts_with("strong:") {
        ("[b]", extract_label(content, "strong"))
    } else if content.starts_with("navigation ") || content == "navigation" || content.starts_with("navigation\"") {
        ("[nav]", extract_label(content, "navigation"))
    } else if content.starts_with("main ") || content == "main" {
        ("[main]", String::new())
    } else if content.starts_with("region ") || content.starts_with("region\"") {
        ("[region]", extract_label(content, "region"))
    } else if content.starts_with("banner ") || content == "banner" {
        ("[banner]", String::new())
    } else if content.starts_with("complementary ") || content.starts_with("complementary\"") {
        ("[aside]", extract_label(content, "complementary"))
    } else if content.starts_with("contentinfo ") || content.starts_with("contentinfo:") {
        ("[footer]", extract_label(content, "contentinfo"))
    } else if content.starts_with("alert ") || content == "alert" {
        ("[alert]", extract_label(content, "alert"))
    } else if content.starts_with("log ") || content == "log" {
        ("[log]", String::new())
    } else if content.starts_with("status ") || content.starts_with("status:") {
        ("[status]", extract_label(content, "status"))
    } else if content.starts_with("figure ") || content == "figure" {
        ("[fig]", String::new())
    } else if content.starts_with("group ") || content.starts_with("group\"") {
        ("[grp]", extract_label(content, "group"))
    } else if content.starts_with("article ") || content == "article" {
        ("[article]", String::new())
    } else if content.starts_with("slider ") || content.starts_with("slider\"") {
        ("[slider]", extract_label(content, "slider"))
    } else if content.starts_with("progressbar ") || content == "progressbar" {
        ("[progress]", String::new())
    } else if content.starts_with("iframe ") || content == "iframe" {
        ("[iframe]", extract_label(content, "iframe"))
    } else if content.starts_with("/url:") {
        return format!("  -> {}", content.trim_start_matches("/url:").trim());
    } else {
        // Keep as-is for unknown types
        return content.to_string();
    };

    // Build output with ref
    let ref_suffix = ref_id.map(|r| format!(" [{}]", r)).unwrap_or_default();

    if label.is_empty() {
        format!("{}{}", symbol, ref_suffix)
    } else {
        format!("{} {}{}", symbol, label, ref_suffix)
    }
}

/// Extract quoted label from element line.
fn extract_label(content: &str, element_type: &str) -> String {
    // Remove element type prefix
    let rest = content.trim_start_matches(element_type).trim();

    // Try to extract quoted content
    if let Some(start) = rest.find('"') {
        if let Some(end) = rest[start + 1..].find('"') {
            return rest[start + 1..start + 1 + end].to_string();
        }
    }

    // For elements with colon content (e.g., "contentinfo: text")
    if let Some(colon_pos) = rest.find(':') {
        let after_colon = rest[colon_pos + 1..].trim();
        // Check if it's not a URL or ref
        if !after_colon.starts_with('/') && !after_colon.starts_with('[') {
            // Remove any trailing ref brackets
            if let Some(bracket_pos) = after_colon.find('[') {
                return after_colon[..bracket_pos].trim().to_string();
            }
            return after_colon.to_string();
        }
    }

    String::new()
}

/// Extract paragraph content.
fn extract_paragraph_content(content: &str) -> String {
    let rest = content.trim_start_matches("paragraph").trim();

    // Check for inline content after colon
    if let Some(colon_pos) = rest.find(':') {
        let after_colon = rest[colon_pos + 1..].trim();
        if !after_colon.is_empty() && !after_colon.starts_with('-') {
            return after_colon.to_string();
        }
    }

    // Check for quoted content
    if let Some(start) = rest.find('"') {
        if let Some(end) = rest[start + 1..].find('"') {
            return rest[start + 1..start + 1 + end].to_string();
        }
    }

    String::new()
}

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
