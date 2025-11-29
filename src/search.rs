//! Search functionality for Playwright MCP snapshots.

use crate::error::MeyerholdError;
use crate::regex::REF_REGEX;
use regex::Regex;
use serde::Serialize;

/// A search result with context.
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    /// Line number (1-indexed)
    pub line_number: usize,
    /// The matching line content
    pub content: String,
    /// Element ref ID if present on this line
    pub ref_id: Option<String>,
}

/// Search for a pattern in snapshot text.
///
/// Returns matching lines with context.
pub fn search(text: &str, pattern: &str, use_regex: bool) -> Result<Vec<SearchResult>, MeyerholdError> {
    let matcher: Box<dyn Fn(&str) -> bool> = if use_regex {
        let re = Regex::new(pattern)?;
        Box::new(move |line: &str| re.is_match(line))
    } else {
        let pattern_lower = pattern.to_lowercase();
        Box::new(move |line: &str| line.to_lowercase().contains(&pattern_lower))
    };

    let mut results = Vec::new();

    for (line_num, line) in text.lines().enumerate() {
        if matcher(line) {
            let ref_id = REF_REGEX
                .captures(line)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string());

            results.push(SearchResult {
                line_number: line_num + 1,
                content: line.to_string(),
                ref_id,
            });
        }
    }

    if results.is_empty() {
        Err(MeyerholdError::SearchNotFound(pattern.to_string()))
    } else {
        Ok(results)
    }
}
