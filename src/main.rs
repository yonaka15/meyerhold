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
//! # View specific ref (path to ref + flat content below)
//! meyerhold snapshot.json --view e407
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
// Binary file detection
// =============================================================================

// Binary file magic bytes
const PNG_MAGIC: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
const JPEG_MAGIC: &[u8] = &[0xFF, 0xD8, 0xFF];
const GIF_MAGIC: &[u8] = b"GIF8";
const RIFF_MAGIC: &[u8] = b"RIFF";
const WEBP_MAGIC: &[u8] = b"WEBP";
const BMP_MAGIC: &[u8] = b"BM";
const TIFF_LE_MAGIC: &[u8] = &[0x49, 0x49, 0x2A, 0x00];
const TIFF_BE_MAGIC: &[u8] = &[0x4D, 0x4D, 0x00, 0x2A];
const PDF_MAGIC: &[u8] = b"%PDF";

enum BinaryType {
    Png,
    Jpeg,
    Gif,
    WebP,
    Bmp,
    Tiff,
    Pdf,
    Unknown,
}

impl BinaryType {
    fn name(&self) -> &'static str {
        match self {
            BinaryType::Png => "PNG image",
            BinaryType::Jpeg => "JPEG image",
            BinaryType::Gif => "GIF image",
            BinaryType::WebP => "WebP image",
            BinaryType::Bmp => "BMP image",
            BinaryType::Tiff => "TIFF image",
            BinaryType::Pdf => "PDF document",
            BinaryType::Unknown => "binary file",
        }
    }
}

fn detect_binary(path: &str) -> Result<Option<BinaryType>, std::io::Error> {
    use std::io::Read;

    let mut file = std::fs::File::open(path)?;
    let mut header = [0u8; 12];
    let bytes_read = file.read(&mut header)?;

    if bytes_read == 0 {
        return Ok(None);
    }

    let header = &header[..bytes_read];

    // Check magic bytes for known formats
    if header.starts_with(PNG_MAGIC) {
        return Ok(Some(BinaryType::Png));
    }
    if header.starts_with(JPEG_MAGIC) {
        return Ok(Some(BinaryType::Jpeg));
    }
    if header.starts_with(GIF_MAGIC) {
        return Ok(Some(BinaryType::Gif));
    }
    if header.starts_with(RIFF_MAGIC) && header.len() >= 12 && &header[8..12] == WEBP_MAGIC {
        return Ok(Some(BinaryType::WebP));
    }
    if header.starts_with(BMP_MAGIC) {
        return Ok(Some(BinaryType::Bmp));
    }
    if header.starts_with(TIFF_LE_MAGIC) || header.starts_with(TIFF_BE_MAGIC) {
        return Ok(Some(BinaryType::Tiff));
    }
    if header.starts_with(PDF_MAGIC) {
        return Ok(Some(BinaryType::Pdf));
    }

    // Check for null bytes (generic binary detection)
    let mut buffer = vec![0u8; 8192];
    let n = file.read(&mut buffer)?;

    if header.contains(&0) || buffer[..n].contains(&0) {
        return Ok(Some(BinaryType::Unknown));
    }

    Ok(None)
}

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
    /// Clickable elements only (excludes disabled)
    Clickable,
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
            CliListType::Clickable => ListType::Clickable,
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

    /// View specific ref (shows path to ref + flat content below)
    #[arg(long)]
    view: Option<String>,

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

    // Check for binary file before attempting text read
    match detect_binary(&args.file) {
        Ok(Some(binary_type)) => {
            eprintln!(
                "ERROR: File '{}' is a {}, not a JSON file",
                args.file,
                binary_type.name()
            );
            eprintln!("Hint: meyerhold expects Playwright MCP snapshot JSON files");
            std::process::exit(EXIT_FILE_ERROR);
        }
        Ok(None) => {}
        Err(e) => {
            eprintln!("ERROR: Failed to read file '{}': {}", args.file, e);
            std::process::exit(EXIT_FILE_ERROR);
        }
    }

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
    } else if let Some(ref_id) = &args.view {
        handle_view(&mh, ref_id, &args);
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
    let clickable = mh.clickable_stats();

    match args.format {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "page_url": summary.page_url,
                "page_title": summary.page_title,
                "tab_count": summary.tab_count,
                "blank_tab_count": summary.blank_tab_count,
                "error_count": summary.error_count,
                "element_count": summary.element_count,
                "clickable": clickable.total,
                "clickable_disabled": clickable.disabled_count,
                "content": summary.content,
                "content_truncated": summary.content_truncated,
            });
            print_json(&json);
        }
        _ => {
            println!("=== Snapshot Summary ===");
            println!();
            println!("Page URL:   {}", summary.page_url);
            println!("Page Title: {}", summary.page_title);
            println!();
            if summary.blank_tab_count > 0 {
                println!(
                    "Tabs:      {} ({} blank)",
                    summary.tab_count, summary.blank_tab_count
                );
            } else {
                println!("Tabs:      {}", summary.tab_count);
            }
            println!("Errors:    {}", summary.error_count);
            println!("Elements:  {}", summary.element_count);
            if clickable.disabled_count > 0 {
                println!(
                    "Clickable: {} ({} disabled excluded)",
                    clickable.total, clickable.disabled_count
                );
            } else {
                println!("Clickable: {}", clickable.total);
            }

            println!();
            println!("Next: --view REF  (show path to ref + content below)");

            // Display content (text, links, buttons, inputs) with hierarchy
            if !summary.content.is_empty() {
                println!();
                println!("--- Content ---");
                print_content_items(&summary.content, args.limit);
                if summary.content_truncated {
                    println!("... (content truncated)");
                }
            }
        }
    }
}

/// Print content items with indentation based on visible ancestors.
fn print_content_items(items: &[meyerhold::ContentItem], limit: Option<usize>) {
    if items.is_empty() {
        return;
    }

    let limit = limit.unwrap_or(usize::MAX);

    for (i, item) in items.iter().enumerate() {
        if i >= limit {
            println!("... (truncated, use --limit to show more)");
            break;
        }

        // depth = number of visible (content item) ancestors
        let depth = get_item_depth(item);
        let indent = "  ".repeat(depth);

        match item {
            meyerhold::ContentItem::Heading { ref_id, label, .. } => {
                if ref_id.is_empty() {
                    println!("{}heading: {}", indent, label);
                } else {
                    println!("{}heading: {} [ref={}]", indent, label, ref_id);
                }
            }
            meyerhold::ContentItem::Text { ref_id, label, .. } => {
                if ref_id.is_empty() {
                    println!("{}text: {}", indent, label);
                } else {
                    println!("{}text: {} [ref={}]", indent, label, ref_id);
                }
            }
            meyerhold::ContentItem::Button { ref_id, label, .. } => {
                if ref_id.is_empty() {
                    println!("{}button: {}", indent, label);
                } else {
                    println!("{}button: {} [ref={}]", indent, label, ref_id);
                }
            }
            meyerhold::ContentItem::Link { ref_id, label, .. } => {
                if ref_id.is_empty() {
                    println!("{}link: {}", indent, label);
                } else {
                    println!("{}link: {} [ref={}]", indent, label, ref_id);
                }
            }
            meyerhold::ContentItem::Input { ref_id, label, .. } => {
                if ref_id.is_empty() {
                    println!("{}input: {}", indent, label);
                } else {
                    println!("{}input: {} [ref={}]", indent, label, ref_id);
                }
            }
        }
    }
}

/// Get depth from ContentItem.
fn get_item_depth(item: &meyerhold::ContentItem) -> usize {
    match item {
        meyerhold::ContentItem::Heading { depth, .. } => *depth,
        meyerhold::ContentItem::Text { depth, .. } => *depth,
        meyerhold::ContentItem::Button { depth, .. } => *depth,
        meyerhold::ContentItem::Link { depth, .. } => *depth,
        meyerhold::ContentItem::Input { depth, .. } => *depth,
    }
}

fn handle_view(mh: &Meyerhold, ref_id: &str, args: &Args) {
    let result = match mh.view(ref_id) {
        Some(r) => r,
        None => {
            eprintln!("ERROR: ref '{}' not found", ref_id);
            std::process::exit(EXIT_SEARCH_ERROR);
        }
    };

    match args.format {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "ref": ref_id,
                "path": result.path,
                "content": result.content,
            });
            print_json(&json);
        }
        _ => {
            // Print path (hierarchy to ref)
            println!("=== Path to [ref={}] ===", ref_id);
            println!();
            for line in &result.path {
                println!("{}", line);
            }

            // Print content below
            if !result.content.is_empty() {
                println!();
                println!("--- Content ---");
                print_content_items(&result.content, args.limit);
            }
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
