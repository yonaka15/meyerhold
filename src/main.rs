//! # meyerhold CLI
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
use meyerhold::{ListType, Meyerhold, MeyerholdError, Section};
use serde::Serialize;
use std::fs;

// =============================================================================
// Exit codes
// =============================================================================

const EXIT_FILE_ERROR: i32 = 1;
const EXIT_JSON_ERROR: i32 = 2;
const EXIT_FORMAT_ERROR: i32 = 3;
const EXIT_SEARCH_ERROR: i32 = 4;
const EXIT_REGEX_ERROR: i32 = 5;

// =============================================================================
// CLI types
// =============================================================================

#[derive(Clone, ValueEnum)]
enum CliSection {
    Tabs,
    Errors,
    Tree,
    Page,
    All,
}

impl From<&CliSection> for Option<Section> {
    fn from(cli: &CliSection) -> Self {
        match cli {
            CliSection::Tabs => Some(Section::Tabs),
            CliSection::Errors => Some(Section::Errors),
            CliSection::Tree => Some(Section::Tree),
            CliSection::Page => Some(Section::Page),
            CliSection::All => None,
        }
    }
}

#[derive(Clone, ValueEnum)]
enum CliListType {
    Buttons,
    Links,
    Inputs,
    Headings,
    Images,
    Text,
    All,
}

impl From<&CliListType> for ListType {
    fn from(cli: &CliListType) -> Self {
        match cli {
            CliListType::Buttons => ListType::Buttons,
            CliListType::Links => ListType::Links,
            CliListType::Inputs => ListType::Inputs,
            CliListType::Headings => ListType::Headings,
            CliListType::Images => ListType::Images,
            CliListType::Text => ListType::Text,
            CliListType::All => ListType::All,
        }
    }
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
    section: Option<CliSection>,

    /// Tree depth to display
    #[arg(short, long)]
    depth: Option<usize>,

    /// Start tree from specific ref
    #[arg(long)]
    from: Option<String>,

    /// List elements by type
    #[arg(short, long)]
    list: Option<CliListType>,

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

    // Parse using library
    let mh = match Meyerhold::from_str(&content) {
        Ok(m) => m,
        Err(e) => {
            let exit_code = match &e {
                MeyerholdError::InvalidJson(_) => EXIT_JSON_ERROR,
                MeyerholdError::MissingContent => EXIT_FORMAT_ERROR,
                _ => EXIT_FORMAT_ERROR,
            };
            eprintln!("ERROR: {}", e);
            std::process::exit(exit_code);
        }
    };

    // Always check for blank tabs (warn on stderr)
    let blank_count = mh.blank_tab_count();
    if blank_count > 0 {
        eprintln!("WARNING: {} blank tab(s) detected (about:blank)", blank_count);
        eprintln!();
    }

    // Route to appropriate handler
    if let Some(search_pattern) = &args.search {
        handle_search(&mh, search_pattern, args.regex, &args);
    } else if let Some(list_type) = &args.list {
        handle_list(&mh, list_type, &args);
    } else if let Some(depth) = args.depth {
        handle_tree(&mh, depth, args.from.as_deref(), &args);
    } else if let Some(section) = &args.section {
        handle_section(&mh, section, &args);
    } else {
        handle_summary(&mh, &args);
    }
}

/// Print JSON with error handling
fn print_json<T: Serialize>(value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("ERROR: Failed to serialize JSON: {}", e);
            std::process::exit(EXIT_JSON_ERROR);
        }
    }
}

fn handle_summary(mh: &Meyerhold, args: &Args) {
    let summary = mh.summary();

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

            // Display all content in DOM order
            if !summary.content.is_empty() {
                println!();
                println!("--- Content (DOM order) ---");
                for item in &summary.content {
                    match item {
                        meyerhold::ContentItem::Heading { label } => {
                            println!("# {}", label);
                        }
                        meyerhold::ContentItem::Text { label } => {
                            println!("  {}", label);
                        }
                        meyerhold::ContentItem::Button { ref_id, label } => {
                            println!("[{}] button: {}", ref_id, label);
                        }
                        meyerhold::ContentItem::Link { ref_id, label } => {
                            println!("[{}] link: {}", ref_id, label);
                        }
                        meyerhold::ContentItem::Input { ref_id, label } => {
                            println!("[{}] input: {}", ref_id, label);
                        }
                    }
                }
                if summary.content_truncated {
                    println!("... (truncated)");
                }
            }

            println!();
            println!("Use --section <tabs|errors|tree|page> for details");
            println!("Use --list <buttons|links|inputs|all> for elements");
            println!("Use --depth N for tree navigation");
        }
    }
}

fn handle_section(mh: &Meyerhold, section: &CliSection, args: &Args) {
    let output = match section {
        CliSection::All => mh.content().to_string(),
        _ => {
            let lib_section: Option<Section> = section.into();
            lib_section
                .and_then(|s| mh.section(s))
                .unwrap_or_default()
        }
    };

    print_output(&output, args);
}

fn handle_tree(mh: &Meyerhold, depth: usize, from_ref: Option<&str>, args: &Args) {
    let output = mh.tree(depth, from_ref);
    print_output(&output, args);
}

fn handle_list(mh: &Meyerhold, list_type: &CliListType, args: &Args) {
    let elements = mh.elements(list_type.into());

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
                    println!(
                        "{:4}: [{}] {} \"{}\"",
                        i + 1,
                        elem.ref_id,
                        elem.element_type,
                        elem.label
                    );
                } else {
                    println!("[{}] {} \"{}\"", elem.ref_id, elem.element_type, elem.label);
                }
            }
            if elements.len() > limit {
                println!(
                    "... and {} more (use --limit to show more)",
                    elements.len() - limit
                );
            }
        }
    }
}

fn handle_search(mh: &Meyerhold, pattern: &str, use_regex: bool, args: &Args) {
    let results = match mh.search(pattern, use_regex) {
        Ok(r) => r,
        Err(MeyerholdError::Regex(e)) => {
            eprintln!("ERROR: Invalid regex: {}", e);
            std::process::exit(EXIT_REGEX_ERROR);
        }
        Err(MeyerholdError::SearchNotFound(p)) => {
            eprintln!("No matches found for pattern: {}", p);
            std::process::exit(EXIT_SEARCH_ERROR);
        }
        Err(e) => {
            eprintln!("ERROR: {}", e);
            std::process::exit(EXIT_SEARCH_ERROR);
        }
    };

    match args.format {
        OutputFormat::Json => {
            let json_results: Vec<_> = results
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "line": r.line_number,
                        "content": r.content,
                        "ref": r.ref_id,
                    })
                })
                .collect();
            print_json(&json_results);
        }
        _ => {
            let limit = args.limit.unwrap_or(results.len());
            for result in results.iter().take(limit) {
                let ref_str = result
                    .ref_id
                    .as_ref()
                    .map(|r| format!(" [{}]", r))
                    .unwrap_or_default();
                if args.line_numbers {
                    println!("{:5}:{}{}", result.line_number, ref_str, result.content.trim());
                } else {
                    println!("{}{}", ref_str, result.content.trim());
                }
            }
            println!();
            println!("Found {} matches", results.len());
        }
    }
}

/// Truncate string at character boundary (UTF-8 safe)
fn truncate_str(s: &str, max_chars: usize) -> String {
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
