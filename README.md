# meyerhold

[![Crates.io](https://img.shields.io/crates/v/meyerhold.svg)](https://crates.io/crates/meyerhold)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Progressive reader for [Playwright MCP](https://github.com/microsoft/playwright-mcp) snapshot JSON files.

Parses the JSON output from `browser_snapshot` tool, which wraps [Playwright](https://playwright.dev/)'s accessibility tree snapshot in MCP format.

## Features

- **Summary view** - Quick overview of page URL, title, tabs, errors, element count, and clickable elements
- **Content preview** - DOM-ordered view of headings, text, buttons, links, and inputs
- **Clickable elements** - List interactive elements with disabled filtering
- **Section extraction** - View tabs, errors, page state, or accessibility tree
- **Element listing** - List buttons, links, inputs, headings, images by type
- **Tree navigation** - Explore DOM structure with depth control
- **Search** - Find elements by text pattern or regex
- **Multiple output formats** - Text, table, or JSON

## Installation

```bash
# From crates.io
cargo install meyerhold

# From source
git clone https://github.com/yonaka15/meyerhold.git
cd meyerhold
cargo install --path .
```

## Usage

```bash
# Summary view (default)
meyerhold snapshot.json

# List clickable elements (excludes disabled)
meyerhold snapshot.json --list clickable

# Show specific sections
meyerhold snapshot.json --section tabs
meyerhold snapshot.json --section errors
meyerhold snapshot.json --section tree
meyerhold snapshot.json --section page

# Tree navigation
meyerhold snapshot.json --depth 2        # Top 2 levels
meyerhold snapshot.json --depth 3 --from e407  # From specific ref

# Element listing
meyerhold snapshot.json --list buttons
meyerhold snapshot.json --list links
meyerhold snapshot.json --list inputs
meyerhold snapshot.json --list all       # All interactive elements

# Search
meyerhold snapshot.json --search "Sign in"
meyerhold snapshot.json --search "button.*submit" --regex

# Output formats
meyerhold snapshot.json --list clickable --format table
meyerhold snapshot.json --list buttons --format json
```

## Example Output

```
=== Snapshot Summary ===

Page URL:   https://example.com/
Page Title: Example Domain

Tabs:      1
Errors:    0
Elements:  150
Clickable: 42 (3 disabled excluded)

--- Content (DOM order) ---
# Welcome to Example
  This is an example page.
[e12] button: Sign In
[e15] link: Learn More
[e23] input: Email address
... (truncated)

Next: --list clickable  (show interactive elements)
      --section <tabs|errors|tree|page> for details
      --search "pattern" to find specific content
```

## Clickable Elements

The `--list clickable` option shows all interactive elements that can be clicked:

- button
- link
- textbox, checkbox, radio, combobox, searchbox
- tab

Elements with `[disabled]` attribute are automatically excluded.

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run with debug output
cargo run -- snapshot.json
```

## License

MIT
