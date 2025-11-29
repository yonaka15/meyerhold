//! # meyerhold
//!
//! Progressive reader for Playwright MCP snapshot JSON files.
//!
//! ## Usage
//!
//! ```bash
//! # Summary view (default)
//! meyerhold snapshot.json
//!
//! # Show specific sections
//! meyerhold snapshot.json --section tabs
//! meyerhold snapshot.json --section errors
//! meyerhold snapshot.json --section tree
//! meyerhold snapshot.json --section page
//!
//! # Tree navigation
//! meyerhold snapshot.json --depth 2        # Top 2 levels
//! meyerhold snapshot.json --depth 3 --from e407  # From specific ref
//!
//! # Element listing
//! meyerhold snapshot.json --list buttons
//! meyerhold snapshot.json --list links
//! meyerhold snapshot.json --list inputs
//! meyerhold snapshot.json --list all       # All interactive elements
//!
//! # Search
//! meyerhold snapshot.json --search "Sign in"
//! meyerhold snapshot.json --search "button.*submit" --regex
//!
//! # Output formats
//! meyerhold snapshot.json --list buttons --format table
//! meyerhold snapshot.json --list buttons --format json
//! ```

use clap::{Parser, ValueEnum};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;
use serde_json::Value;
use std::fs;

// =============================================================================
// Constants
// =============================================================================

// Exit codes for different error types
const EXIT_FILE_ERROR: i32 = 1;
const EXIT_JSON_ERROR: i32 = 2;
const EXIT_FORMAT_ERROR: i32 = 3;
const EXIT_SEARCH_ERROR: i32 = 4;
const EXIT_REGEX_ERROR: i32 = 5;

// Snapshot section markers (Playwright MCP format)
const SECTION_TABS: &str = "### Open tabs";
const SECTION_ERRORS: &str = "### New console messages";
const SECTION_PAGE_STATE: &str = "### Page state";
const SECTION_TREE_START: &str = "```yaml";
const SECTION_TREE_END: &str = "```";
const SECTION_END: &str = "###";

// Page state field prefixes
const FIELD_PAGE_URL: &str = "- Page URL:";
const FIELD_PAGE_TITLE: &str = "- Page Title:";
const FIELD_PAGE_SNAPSHOT: &str = "- Page Snapshot:";

// Error/warning markers in console messages
const MARKER_ERROR: &str = "[ERROR]";
const MARKER_WARNING: &str = "[WARNING]";

// Interactive element types for extraction
const ELEM_BUTTON: &str = "button";
const ELEM_LINK: &str = "link";
const ELEM_TEXTBOX: &str = "textbox";
const ELEM_CHECKBOX: &str = "checkbox";
const ELEM_RADIO: &str = "radio";
const ELEM_COMBOBOX: &str = "combobox";
const ELEM_SEARCHBOX: &str = "searchbox";
const ELEM_MENUITEM: &str = "menuitem";
const ELEM_TAB: &str = "tab";
const ELEM_OPTION: &str = "option";
const ELEM_HEADING: &str = "heading";
const ELEM_IMG: &str = "img";

// =============================================================================
// Pre-compiled regex patterns
// =============================================================================

static REF_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[ref=([a-zA-Z0-9-]+)\]").unwrap());
static QUOTE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?:button|link|textbox|heading|img|checkbox|radio|menuitem|tab|option|combobox)\s*"([^"]+)""#).unwrap()
});
static SIMPLE_QUOTE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#""([^"]+)""#).unwrap());

#[derive(Clone, ValueEnum)]
enum Section {
    Tabs,
    Errors,
    Tree,
    Page,
    All,
}

#[derive(Clone, ValueEnum)]
enum ListType {
    Buttons,
    Links,
    Inputs,
    Headings,
    Images,
    Text,
    All,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Text,
    Table,
    Json,
}

#[derive(Parser)]
#[command(name = "meyerhold")]
#[command(about = "Progressive reader for Playwright MCP snapshots")]
#[command(version)]
struct Args {
    /// Path to snapshot JSON file
    file: String,

    /// Show specific section
    #[arg(short, long)]
    section: Option<Section>,

    /// Tree depth to display
    #[arg(short, long)]
    depth: Option<usize>,

    /// Start tree from specific ref
    #[arg(long)]
    from: Option<String>,

    /// List elements by type
    #[arg(short, long)]
    list: Option<ListType>,

    /// Search pattern
    #[arg(long)]
    search: Option<String>,

    /// Treat search pattern as regex
    #[arg(long)]
    regex: bool,

    /// Output format
    #[arg(short, long, default_value = "text")]
    format: OutputFormat,

    /// Show line numbers
    #[arg(short = 'n', long)]
    line_numbers: bool,

    /// Limit output lines
    #[arg(long)]
    limit: Option<usize>,
}

#[derive(Serialize)]
struct Element {
    ref_id: String,
    element_type: String,
    label: String,
    depth: usize,
}

#[derive(Serialize)]
struct SnapshotSummary {
    page_url: String,
    page_title: String,
    tab_count: usize,
    blank_tab_count: usize,
    error_count: usize,
    element_count: usize,
}

fn main() {
    let args = Args::parse();

    // Read file
    let content = match fs::read_to_string(&args.file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("ERROR: Failed to read file '{}': {}", args.file, e);
            std::process::exit(EXIT_FILE_ERROR);
        }
    };

    // Validate non-empty content
    if content.trim().is_empty() {
        eprintln!("ERROR: File '{}' is empty", args.file);
        std::process::exit(EXIT_FILE_ERROR);
    }

    // Parse JSON
    let json: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("ERROR: Invalid JSON: {}", e);
            std::process::exit(EXIT_JSON_ERROR);
        }
    };

    // Extract snapshot text
    let snapshot_text = match extract_snapshot_text(&json) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("ERROR: {}", e);
            std::process::exit(EXIT_FORMAT_ERROR);
        }
    };

    // Always check for blank tabs (warn on stderr)
    check_blank_tabs(&snapshot_text);

    // Route to appropriate handler
    if let Some(search_pattern) = &args.search {
        handle_search(&snapshot_text, search_pattern, args.regex, &args);
    } else if let Some(list_type) = &args.list {
        handle_list(&snapshot_text, list_type, &args);
    } else if let Some(depth) = args.depth {
        handle_tree(&snapshot_text, depth, args.from.as_deref(), &args);
    } else if let Some(section) = &args.section {
        handle_section(&snapshot_text, section, &args);
    } else {
        // Default: show summary
        handle_summary(&snapshot_text, &args);
    }
}

/// Check for blank tabs and print warning to stderr
fn check_blank_tabs(text: &str) {
    let tab_section = extract_section(text, SECTION_TABS, SECTION_END);
    let blank_count = tab_section
        .lines()
        .filter(|l| l.starts_with("- ") && l.contains("(about:blank)"))
        .count();

    if blank_count > 0 {
        eprintln!(
            "WARNING: {} blank tab(s) detected (about:blank)",
            blank_count
        );
        eprintln!();
    }
}

fn extract_snapshot_text(json: &Value) -> Result<String, String> {
    json.get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("text"))
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Could not find snapshot text (expected .content[0].text)".to_string())
}

/// Print JSON with error handling (avoids unwrap panics)
fn print_json<T: Serialize>(value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("ERROR: Failed to serialize JSON: {}", e);
            std::process::exit(EXIT_JSON_ERROR);
        }
    }
}

fn handle_summary(text: &str, args: &Args) {
    let summary = parse_summary(text);

    match args.format {
        OutputFormat::Json => {
            print_json(&summary);
        }
        _ => {
            println!("=== Snapshot Summary ===");
            println!();
            println!("Page URL:   {}", summary.page_url);
            println!("Page Title: {}", summary.page_title);
            println!();
            if summary.blank_tab_count > 0 {
                println!(
                    "Tabs:     {} ({} blank)",
                    summary.tab_count, summary.blank_tab_count
                );
            } else {
                println!("Tabs:     {}", summary.tab_count);
            }
            println!("Errors:   {}", summary.error_count);
            println!("Elements: {}", summary.element_count);
            println!();
            println!("Use --section <tabs|errors|tree|page> for details");
            println!("Use --list <buttons|links|inputs|all> for elements");
            println!("Use --depth N for tree navigation");
        }
    }
}

fn parse_summary(text: &str) -> SnapshotSummary {
    let mut page_url = String::new();
    let mut page_title = String::new();
    let mut error_count = 0;

    let element_count = REF_REGEX.find_iter(text).count();

    for line in text.lines() {
        if line.starts_with(FIELD_PAGE_URL) {
            page_url = line.trim_start_matches(FIELD_PAGE_URL).trim().to_string();
        } else if line.starts_with(FIELD_PAGE_TITLE) {
            page_title = line.trim_start_matches(FIELD_PAGE_TITLE).trim().to_string();
        } else if line.contains(MARKER_ERROR) || line.contains(MARKER_WARNING) {
            error_count += 1;
        }
    }

    // Tab counting from dedicated section
    let tab_section = extract_section(text, SECTION_TABS, SECTION_END);
    let tab_lines: Vec<&str> = tab_section.lines().filter(|l| l.starts_with("- ")).collect();
    let tab_count = tab_lines.len();
    let blank_tab_count = tab_lines.iter().filter(|l| l.contains("(about:blank)")).count();

    SnapshotSummary {
        page_url,
        page_title,
        tab_count,
        blank_tab_count,
        error_count,
        element_count,
    }
}

fn handle_section(text: &str, section: &Section, args: &Args) {
    let output = match section {
        Section::Tabs => extract_section(text, SECTION_TABS, SECTION_END),
        Section::Errors => extract_section(text, SECTION_ERRORS, SECTION_END),
        Section::Page => extract_page_state(text),
        Section::Tree => extract_section(text, SECTION_TREE_START, SECTION_TREE_END),
        Section::All => text.to_string(),
    };

    print_output(&output, args);
}

fn extract_section(text: &str, start_marker: &str, end_marker: &str) -> String {
    let mut result = Vec::new();
    let mut in_section = false;

    for line in text.lines() {
        if line.contains(start_marker) {
            in_section = true;
            if !start_marker.starts_with("```") {
                result.push(line.to_string());
            }
            continue;
        }

        if in_section {
            if line.starts_with(end_marker) && line != start_marker {
                break;
            }
            result.push(line.to_string());
        }
    }

    result.join("\n")
}

fn extract_page_state(text: &str) -> String {
    let mut result = Vec::new();
    let mut in_state = false;

    for line in text.lines() {
        if line.contains(SECTION_PAGE_STATE) {
            in_state = true;
            continue;
        }

        if in_state {
            if line.starts_with(FIELD_PAGE_SNAPSHOT) {
                result.push(line.to_string());
                break;
            }
            result.push(line.to_string());
        }
    }

    result.join("\n")
}

fn handle_tree(text: &str, max_depth: usize, from_ref: Option<&str>, args: &Args) {
    let tree_text = extract_section(text, SECTION_TREE_START, SECTION_TREE_END);
    let mut output_lines = Vec::new();
    let mut found_from = from_ref.is_none();
    let mut base_indent = 0;

    for line in tree_text.lines() {
        // Calculate depth from leading spaces
        let indent = line.len() - line.trim_start().len();
        let depth = indent / 2;

        // If we have a from_ref, look for it
        if let Some(ref_id) = from_ref {
            if !found_from {
                if line.contains(&format!("[ref={}]", ref_id)) {
                    found_from = true;
                    base_indent = indent;
                } else {
                    continue;
                }
            } else {
                // Check if we've exited the subtree (new element at same or lower indent)
                let is_new_element = line.trim().starts_with('-') || line.contains("[ref=");
                if indent <= base_indent && is_new_element && !output_lines.is_empty() {
                    break;
                }
            }
        }

        // Filter by depth
        let effective_depth = if from_ref.is_some() {
            (indent.saturating_sub(base_indent)) / 2
        } else {
            depth
        };

        if effective_depth < max_depth {
            output_lines.push(line.to_string());
        }
    }

    let output = output_lines.join("\n");
    print_output(&output, args);
}

fn handle_list(text: &str, list_type: &ListType, args: &Args) {
    let elements = extract_elements(text, list_type);

    match args.format {
        OutputFormat::Json => {
            print_json(&elements);
        }
        OutputFormat::Table => {
            println!("{:<10} {:<15} {}", "REF", "TYPE", "LABEL");
            println!("{}", "-".repeat(60));
            let limit = args.limit.unwrap_or(elements.len());
            for elem in elements.iter().take(limit) {
                let label = truncate_str(&elem.label, 40);
                println!("{:<10} {:<15} {}", elem.ref_id, elem.element_type, label);
            }
            println!();
            println!("Total: {} elements", elements.len());
        }
        OutputFormat::Text => {
            let limit = args.limit.unwrap_or(elements.len());
            for (i, elem) in elements.iter().take(limit).enumerate() {
                if args.line_numbers {
                    println!("{:4}: [{}] {} \"{}\"", i + 1, elem.ref_id, elem.element_type, elem.label);
                } else {
                    println!("[{}] {} \"{}\"", elem.ref_id, elem.element_type, elem.label);
                }
            }
            if elements.len() > limit {
                println!("... and {} more (use --limit to show more)", elements.len() - limit);
            }
        }
    }
}

fn extract_elements(text: &str, list_type: &ListType) -> Vec<Element> {
    let mut elements = Vec::new();

    // Type patterns for matching (small set, Vec is fine for iteration)
    let type_patterns: &[&str] = match list_type {
        ListType::Buttons => &[ELEM_BUTTON],
        ListType::Links => &[ELEM_LINK],
        ListType::Inputs => &[ELEM_TEXTBOX, ELEM_CHECKBOX, ELEM_RADIO, ELEM_COMBOBOX, ELEM_SEARCHBOX],
        ListType::Headings => &[ELEM_HEADING],
        ListType::Images => &[ELEM_IMG],
        ListType::Text => &["__text__"], // Special marker for text extraction
        ListType::All => &[
            ELEM_BUTTON, ELEM_LINK, ELEM_TEXTBOX, ELEM_CHECKBOX, ELEM_RADIO,
            ELEM_COMBOBOX, ELEM_SEARCHBOX, ELEM_MENUITEM, ELEM_TAB, ELEM_OPTION,
        ],
    };

    // Special handling for text extraction
    let extract_text_only = matches!(list_type, ListType::Text);

    for line in text.lines() {
        let trimmed = line.trim().trim_start_matches("- ");

        // Text extraction mode: look for "text:" lines and text content
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

        // Check if line matches any type pattern
        let matched_type = type_patterns
            .iter()
            .find(|&&p| trimmed.starts_with(p) || trimmed.starts_with(&format!("'{}", p)));

        if let Some(&elem_type) = matched_type {
            if let Some(caps) = REF_REGEX.captures(line) {
                let ref_id = caps.get(1).unwrap().as_str().to_string();

                // Extract label (text in quotes after element type)
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

fn extract_label(line: &str) -> String {
    // Match quoted string after element type (using pre-compiled regex)
    if let Some(caps) = QUOTE_REGEX.captures(line) {
        return caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
    }

    // Fallback: extract first quoted string (using pre-compiled regex)
    if let Some(caps) = SIMPLE_QUOTE_REGEX.captures(line) {
        return caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
    }

    String::new()
}

fn handle_search(text: &str, pattern: &str, use_regex: bool, args: &Args) {
    let matcher: Box<dyn Fn(&str) -> bool> = if use_regex {
        let re = match Regex::new(pattern) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("ERROR: Invalid regex: {}", e);
                std::process::exit(EXIT_REGEX_ERROR);
            }
        };
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

            results.push((line_num + 1, line.to_string(), ref_id));
        }
    }

    if results.is_empty() {
        eprintln!("No matches found for pattern: {}", pattern);
        std::process::exit(EXIT_SEARCH_ERROR);
    }

    match args.format {
        OutputFormat::Json => {
            let json_results: Vec<_> = results
                .iter()
                .map(|(num, line, ref_id)| {
                    serde_json::json!({
                        "line": num,
                        "content": line,
                        "ref": ref_id,
                    })
                })
                .collect();
            print_json(&json_results);
        }
        _ => {
            let limit = args.limit.unwrap_or(results.len());
            for (line_num, line, ref_id) in results.iter().take(limit) {
                let ref_str = ref_id.as_ref().map(|r| format!(" [{}]", r)).unwrap_or_default();
                if args.line_numbers {
                    println!("{:5}:{}{}", line_num, ref_str, line.trim());
                } else {
                    println!("{}{}", ref_str, line.trim());
                }
            }
            println!();
            println!("Found {} matches", results.len());
        }
    }
}

/// Truncate string at character boundary (UTF-8 safe, allocation-efficient)
fn truncate_str(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        return s.to_string();
    }

    // Find byte position of the (max_chars - 3)th character
    let truncate_at = max_chars.saturating_sub(3);
    let end_pos = s
        .char_indices()
        .nth(truncate_at)
        .map(|(pos, _)| pos)
        .unwrap_or(s.len());

    format!("{}...", &s[..end_pos])
}

fn print_output(text: &str, args: &Args) {
    let limit = args.limit.unwrap_or(usize::MAX);

    for (i, line) in text.lines().take(limit).enumerate() {
        if args.line_numbers {
            println!("{:5}: {}", i + 1, line);
        } else {
            println!("{}", line);
        }
    }
}

#[cfg(test)]
mod tests;
