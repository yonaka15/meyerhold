//! Pre-compiled regex patterns.

use std::sync::LazyLock;

use regex::Regex;

/// Matches element reference IDs like [ref=e407]
pub static REF_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[ref=([a-zA-Z0-9-]+)\]").unwrap());

/// Matches quoted strings after element types
pub static QUOTE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:button|link|textbox|heading|img|checkbox|radio|menuitem|tab|option|combobox)\s*"([^"]+)""#).unwrap()
});

/// Matches any quoted string
pub static SIMPLE_QUOTE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#""([^"]+)""#).unwrap());
