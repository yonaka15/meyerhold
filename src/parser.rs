//! Core parsing logic for Playwright MCP snapshots.

use crate::error::MeyerholdError;
use serde_json::Value;

/// Extract snapshot text from Playwright MCP JSON structure.
///
/// Expects JSON with structure: `{ "content": [{ "text": "..." }] }`
pub fn extract_snapshot_text(json: &Value) -> Result<String, MeyerholdError> {
    json.get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("text"))
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
        .ok_or(MeyerholdError::MissingContent)
}

/// Parse JSON string into Value.
pub fn parse_json(json_str: &str) -> Result<Value, MeyerholdError> {
    serde_json::from_str(json_str).map_err(|e| MeyerholdError::InvalidJson(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_snapshot_text() {
        let json = serde_json::json!({
            "content": [{ "text": "hello world" }]
        });
        assert_eq!(extract_snapshot_text(&json).unwrap(), "hello world");
    }

    #[test]
    fn test_missing_content() {
        let json = serde_json::json!({});
        assert!(extract_snapshot_text(&json).is_err());
    }
}
