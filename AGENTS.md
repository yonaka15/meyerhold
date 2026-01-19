# meyerhold

Progressive reader for Playwright MCP snapshot JSON files. Parses `.content[0].text` structure from browser snapshots.

## Critical Guardrails

- **Snapshot format assumption**: Tool expects Playwright MCP output structure (`{"content":[{"text":"..."}]}`). Raw YAML/HTML won't work.
- **Read-only tool**: Analyzes snapshots only. Does not interact with browsers or modify files.
- **Exit codes matter**: Non-zero exits indicate specific errors (1=file, 2=JSON, 3=format, 4=search, 5=regex).

## Playwright MCP Tools That Return Snapshots

Most action tools return snapshots after execution. Source: [microsoft/playwright mcp/browser/tools](https://github.com/microsoft/playwright/tree/main/packages/playwright/src/mcp/browser/tools)

| Tool | Snapshot | Notes |
|------|----------|-------|
| `browser_snapshot` | Always | Explicit snapshot capture |
| `browser_click` | Always | After click completes |
| `browser_fill` | Always | After form fill |
| `browser_select_option` | Always | After selection |
| `browser_press_key` | Always | After key press |
| `browser_type` | Conditional | When `slowly=true` or `submit=true` |
| `browser_mouse_click_xy` | Always | After XY coordinate click |
| `browser_mouse_drag` | Always | After drag operation |
| `browser_hover` | Always | After hover |
| `browser_navigate` | Always | After navigation |
| `browser_go_back` | Always | After back navigation |
| `browser_file_upload` | Always | After file upload |
| `browser_handle_dialog` | Always | After dialog handling |
| `browser_tabs` | Varies | Tab operations may include snapshot |
| `browser_take_screenshot` | No | Returns image, not accessibility snapshot |
| `browser_pdf_save` | No | Returns PDF file |
| `browser_network_requests` | No | Returns network data only |

**Output format differences**:
- `browser_snapshot`: Includes `### Open tabs` section
- Action tools (click, fill, etc.): Include `### Ran Playwright code` section with executed code

## Core Workflow (80% of use cases)

### 1. Quick Page Analysis

```bash
# Get summary with full tree (URL, title, counts, and page structure)
# Automatically warns if blank tabs (about:blank) detected
meyerhold snapshot.json

# Check for errors first
meyerhold snapshot.json --section errors

# List clickable elements (buttons, links, inputs, tabs - excludes disabled)
meyerhold snapshot.json --list clickable

# List ALL interactive elements (including disabled)
meyerhold snapshot.json --list all --format table
```

**Blank Tab Warning**: If blank tabs are detected, a warning is printed to stderr:
```
WARNING: 3 blank tab(s) detected (about:blank)
```

### 2. Find Specific Elements

```bash
# Search by text (case-insensitive)
meyerhold snapshot.json --search "Sign in"

# Search with regex
meyerhold snapshot.json --search "button.*submit" --regex

# List by type
meyerhold snapshot.json --list buttons
meyerhold snapshot.json --list links
meyerhold snapshot.json --list inputs
```

### 3. DOM Navigation

```bash
# Top-level structure (depth 2)
meyerhold snapshot.json --depth 2

# Subtree from specific ref
meyerhold snapshot.json --depth 3 --from e407

# View specific ref (path to ref + flat content below)
meyerhold snapshot.json --view e407
```

### 4. Export for Further Processing

```bash
# JSON output for parsing
meyerhold snapshot.json --list links --format json

# Summary as JSON
meyerhold snapshot.json --format json
```

## Common Alternatives

- **Need tabs info**: `--section tabs` (not in summary view)
- **Large output**: Add `--limit N` to truncate results
- **Line references**: Add `-n` for line numbers
- **Raw tree access**: `--section tree` for full YAML block

## Anti-patterns to Avoid

- **Parsing raw snapshot with jq**: Use meyerhold instead → Better element extraction and search
- **Grepping snapshot files directly**: `--search` handles context and refs correctly
- **Reading full tree for element finding**: `--list <type>` is faster and cleaner
- **Ignoring exit codes**: Check return value when scripting (especially for search)

## Development

### Building

```bash
cargo build           # Debug build
cargo build --release # Release build
cargo install --path . # Install locally
```

### Running Tests

```bash
cargo test            # Run all tests
cargo test -- --nocapture  # Show output
```

### Project Structure

```
├── Cargo.toml           # Package manifest
├── src/
│   └── main.rs          # CLI entry point and all logic
├── tests/
│   └── integration.rs   # Integration tests (24 tests)
└── testdata/
    └── example.json     # Sample snapshot for tests
```

### Key Functions (src/main.rs)

- `extract_snapshot_text()`: Parse `.content[0].text` from JSON
- `extract_elements()`: Find elements by type (buttons, links, etc.)
- `handle_tree()`: DOM navigation with depth/from-ref support
- `handle_search()`: Text/regex pattern matching

### Dependencies

```toml
serde_json = "1.0"
serde = { version = "1.0", features = ["derive"] }
regex = "1.10"
clap = { version = "4", features = ["derive"] }
thiserror = "1.0"
```

## Quick Reference

| Command | Purpose |
|---------|---------|
| `meyerhold FILE` | Summary with full tree |
| `--list clickable` | Clickable elements only (excludes disabled) |
| `--list buttons\|links\|inputs\|all` | List elements by type |
| `--section tabs\|errors\|tree\|page` | Show section |
| `--depth N` | Tree navigation |
| `--view REF` | Path to ref + flat content below |
| `--search "pattern"` | Find text |
| `--search "pattern" --regex` | Regex search |
| `--format json\|table\|text` | Output format |
| `--limit N` | Truncate output |
| `-n` | Line numbers |

## TODOs

None currently tracked.
