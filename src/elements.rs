//! Element extraction and types.

use crate::constants::*;
use crate::regex::{QUOTE_REGEX, REF_REGEX, SIMPLE_QUOTE_REGEX};
use serde::Serialize;

/// Type of elements to list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListType {
    Buttons,
    Links,
    Inputs,
    Headings,
    Images,
    Text,
    All,
}

/// An interactive element extracted from snapshot.
#[derive(Debug, Clone, Serialize)]
pub struct Element {
    /// Element reference ID (e.g., "e407")
    pub ref_id: String,
    /// Element type (e.g., "button", "link")
    pub element_type: String,
    /// Element label/text content
    pub label: String,
    /// Nesting depth in the tree
    pub depth: usize,
}

/// Extract elements of specified type from snapshot text.
pub fn extract_elements(text: &str, list_type: ListType) -> Vec<Element> {
    let mut elements = Vec::new();

    let type_patterns: &[&str] = match list_type {
        ListType::Buttons => &[ELEM_BUTTON],
        ListType::Links => &[ELEM_LINK],
        ListType::Inputs => &[
            ELEM_TEXTBOX,
            ELEM_CHECKBOX,
            ELEM_RADIO,
            ELEM_COMBOBOX,
            ELEM_SEARCHBOX,
        ],
        ListType::Headings => &[ELEM_HEADING],
        ListType::Images => &[ELEM_IMG],
        ListType::Text => &["__text__"],
        ListType::All => &[
            ELEM_BUTTON,
            ELEM_LINK,
            ELEM_TEXTBOX,
            ELEM_CHECKBOX,
            ELEM_RADIO,
            ELEM_COMBOBOX,
            ELEM_SEARCHBOX,
            ELEM_MENUITEM,
            ELEM_TAB,
            ELEM_OPTION,
        ],
    };

    let extract_text_only = matches!(list_type, ListType::Text);

    for line in text.lines() {
        let trimmed = line.trim().trim_start_matches("- ");

        if extract_text_only {
            if trimmed.starts_with("text:") || trimmed.starts_with("- text:") {
                let content = trimmed
                    .trim_start_matches("- ")
                    .trim_start_matches("text:")
                    .trim()
                    .trim_matches('"')
                    .to_string();
                if !content.is_empty() {
                    let indent = line.len() - line.trim_start().len();
                    elements.push(Element {
                        ref_id: String::new(),
                        element_type: "text".to_string(),
                        label: content,
                        depth: indent / 2,
                    });
                }
            }
            continue;
        }

        let matched_type = type_patterns
            .iter()
            .find(|&&p| trimmed.starts_with(p) || trimmed.starts_with(&format!("'{}", p)));

        if let Some(&elem_type) = matched_type {
            if let Some(caps) = REF_REGEX.captures(line) {
                let ref_id = caps.get(1).unwrap().as_str().to_string();
                let label = extract_label(trimmed);
                let indent = line.len() - line.trim_start().len();
                let depth = indent / 2;

                elements.push(Element {
                    ref_id,
                    element_type: elem_type.to_string(),
                    label,
                    depth,
                });
            }
        }
    }

    elements
}

/// Extract label from element line.
fn extract_label(line: &str) -> String {
    if let Some(caps) = QUOTE_REGEX.captures(line) {
        return caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
    }

    if let Some(caps) = SIMPLE_QUOTE_REGEX.captures(line) {
        return caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
    }

    String::new()
}
