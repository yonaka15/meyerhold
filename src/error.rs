//! Error types for the meyerhold library.

use thiserror::Error;

/// Errors that can occur when working with Playwright MCP snapshots.
#[derive(Debug, Error)]
pub enum MeyerholdError {
    /// JSON parsing or structure error
    #[error("Invalid JSON: {0}")]
    InvalidJson(String),

    /// Missing expected content in snapshot
    #[error("Missing snapshot content: expected .content[0].text")]
    MissingContent,

    /// Invalid regex pattern
    #[error("Invalid regex pattern: {0}")]
    Regex(#[from] regex::Error),

    /// Search pattern not found
    #[error("No matches found for pattern: {0}")]
    SearchNotFound(String),
}
