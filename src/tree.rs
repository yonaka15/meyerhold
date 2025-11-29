//! Tree navigation for Playwright MCP snapshots.

use crate::constants::{SECTION_TREE_END, SECTION_TREE_START};
use crate::summary::extract_section;

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
