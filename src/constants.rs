//! Constants used across the meyerhold library.

// Snapshot section markers (Playwright MCP format)
pub const SECTION_TABS: &str = "### Open tabs";
pub const SECTION_ERRORS: &str = "### New console messages";
pub const SECTION_PAGE_STATE: &str = "### Page state";
pub const SECTION_TREE_START: &str = "```yaml";
pub const SECTION_TREE_END: &str = "```";
pub const SECTION_END: &str = "###";

// Page state field prefixes
pub const FIELD_PAGE_URL: &str = "- Page URL:";
pub const FIELD_PAGE_TITLE: &str = "- Page Title:";
pub const FIELD_PAGE_SNAPSHOT: &str = "- Page Snapshot:";

// Error/warning markers in console messages
pub const MARKER_ERROR: &str = "[ERROR]";
pub const MARKER_WARNING: &str = "[WARNING]";

// Interactive element types for extraction
pub const ELEM_BUTTON: &str = "button";
pub const ELEM_LINK: &str = "link";
pub const ELEM_TEXTBOX: &str = "textbox";
pub const ELEM_CHECKBOX: &str = "checkbox";
pub const ELEM_RADIO: &str = "radio";
pub const ELEM_COMBOBOX: &str = "combobox";
pub const ELEM_SEARCHBOX: &str = "searchbox";
pub const ELEM_MENUITEM: &str = "menuitem";
pub const ELEM_TAB: &str = "tab";
pub const ELEM_OPTION: &str = "option";
pub const ELEM_HEADING: &str = "heading";
pub const ELEM_IMG: &str = "img";
