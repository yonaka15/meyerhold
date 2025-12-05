//! # meyerhold
//!
//! Progressive reader for Playwright MCP snapshot JSON files.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use meyerhold::{Meyerhold, ListType};
//!
//! // Parse from JSON string
//! let json_str = r#"{"content":[{"text":"snapshot content..."}]}"#;
//! let mh = Meyerhold::from_str(json_str)?;
//!
//! // Get summary
//! let summary = mh.summary();
//! println!("URL: {}", summary.page_url);
//!
//! // List elements
//! let buttons = mh.elements(ListType::Buttons);
//! for btn in buttons {
//!     println!("[{}] {}", btn.ref_id, btn.label);
//! }
//!
//! // Search
//! let results = mh.search("Sign in", false)?;
//! # Ok::<(), meyerhold::MeyerholdError>(())
//! ```

mod constants;
mod elements;
mod error;
mod parser;
mod regex;
mod search;
mod summary;
mod tree;

// Public re-exports
pub use elements::{Element, ListType};
pub use error::MeyerholdError;
pub use search::SearchResult;
pub use summary::{ContentItem, SnapshotSummary, SummaryElement, DEFAULT_TEXT_CHAR_LIMIT};

use serde_json::Value;

/// Section types for extraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// Open tabs section
    Tabs,
    /// Console errors/warnings section
    Errors,
    /// YAML tree section
    Tree,
    /// Page state section (URL, title, etc.)
    Page,
}

/// Main interface for working with Playwright MCP snapshots.
///
/// Provides methods for parsing, summarizing, and extracting data from snapshots.
#[derive(Debug, Clone)]
pub struct Meyerhold {
    /// The extracted snapshot text content
    snapshot_text: String,
}

impl Meyerhold {
    /// Create from a JSON string.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use meyerhold::Meyerhold;
    ///
    /// let json = r#"{"content":[{"text":"..."}]}"#;
    /// let mh = Meyerhold::from_str(json)?;
    /// # Ok::<(), meyerhold::MeyerholdError>(())
    /// ```
    pub fn from_str(json_str: &str) -> Result<Self, MeyerholdError> {
        let json = parser::parse_json(json_str)?;
        Self::from_json(&json)
    }

    /// Create from a parsed JSON Value.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use meyerhold::Meyerhold;
    /// use serde_json::json;
    ///
    /// let json = json!({"content":[{"text":"..."}]});
    /// let mh = Meyerhold::from_json(&json)?;
    /// # Ok::<(), meyerhold::MeyerholdError>(())
    /// ```
    pub fn from_json(json: &Value) -> Result<Self, MeyerholdError> {
        let snapshot_text = parser::extract_snapshot_text(json)?;
        Ok(Self { snapshot_text })
    }

    /// Create directly from snapshot text (already extracted).
    ///
    /// Use this when you already have the text content from `.content[0].text`.
    pub fn from_text(text: impl Into<String>) -> Self {
        Self {
            snapshot_text: text.into(),
        }
    }

    /// Get the raw snapshot text content.
    pub fn content(&self) -> &str {
        &self.snapshot_text
    }

    /// Get a summary of the snapshot.
    ///
    /// Returns page URL, title, tab count, error count, and element count.
    pub fn summary(&self) -> SnapshotSummary {
        summary::parse_summary(&self.snapshot_text)
    }

    /// Extract elements of the specified type.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use meyerhold::{Meyerhold, ListType};
    ///
    /// let mh = Meyerhold::from_str(json_str)?;
    /// let buttons = mh.elements(ListType::Buttons);
    /// let all = mh.elements(ListType::All);
    /// ```
    pub fn elements(&self, list_type: ListType) -> Vec<Element> {
        elements::extract_elements(&self.snapshot_text, list_type)
    }

    /// Search for a pattern in the snapshot.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The search pattern
    /// * `use_regex` - If true, treat pattern as regex; otherwise case-insensitive text search
    ///
    /// # Returns
    ///
    /// Returns `Err(MeyerholdError::SearchNotFound)` if no matches found.
    pub fn search(&self, pattern: &str, use_regex: bool) -> Result<Vec<SearchResult>, MeyerholdError> {
        search::search(&self.snapshot_text, pattern, use_regex)
    }

    /// Extract a specific section from the snapshot.
    ///
    /// Returns `None` if the section is empty.
    pub fn section(&self, section: Section) -> Option<String> {
        let result = match section {
            Section::Tabs => {
                summary::extract_section(&self.snapshot_text, constants::SECTION_TABS, constants::SECTION_END)
            }
            Section::Errors => {
                summary::extract_section(&self.snapshot_text, constants::SECTION_ERRORS, constants::SECTION_END)
            }
            Section::Tree => {
                summary::extract_section(&self.snapshot_text, constants::SECTION_TREE_START, constants::SECTION_TREE_END)
            }
            Section::Page => summary::extract_page_state(&self.snapshot_text),
        };

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    /// Navigate the DOM tree with depth and optional starting point.
    ///
    /// # Arguments
    ///
    /// * `depth` - Maximum depth to display
    /// * `from_ref` - Optional ref ID to start from (e.g., "e407")
    pub fn tree(&self, depth: usize, from_ref: Option<&str>) -> String {
        tree::get_tree(&self.snapshot_text, depth, from_ref)
    }

    /// Count blank tabs (about:blank) in the snapshot.
    pub fn blank_tab_count(&self) -> usize {
        summary::count_blank_tabs(&self.snapshot_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_json() {
        let json = serde_json::json!({
            "content": [{ "text": "test content" }]
        });
        let mh = Meyerhold::from_json(&json).unwrap();
        assert_eq!(mh.content(), "test content");
    }

    #[test]
    fn test_from_text() {
        let mh = Meyerhold::from_text("direct text");
        assert_eq!(mh.content(), "direct text");
    }
}
