# meyerhold

[![Crates.io](https://img.shields.io/crates/v/meyerhold.svg)](https://crates.io/crates/meyerhold)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Progressive reader for [Playwright MCP](https://github.com/microsoft/playwright-mcp) snapshot JSON files.

Parses the JSON output from `browser_snapshot` tool, which wraps [Playwright](https://playwright.dev/)'s accessibility tree snapshot in MCP format.

## Features

- **Summary view** - Quick overview of page URL, title, tabs, errors, and element count
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
meyerhold snapshot.json --list buttons --format table
meyerhold snapshot.json --list buttons --format json
```

## Example Output

```
=== Snapshot Summary ===

Page URL:   https://example.com/
Page Title: Example Domain

Tabs:     1
Errors:   0
Elements: 42

Use --section <tabs|errors|tree|page> for details
Use --list <buttons|links|inputs|all> for elements
Use --depth N for tree navigation
```

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
