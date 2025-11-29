//! CLI-specific tests for meyerhold binary.

use super::*;

const SAMPLE_SNAPSHOT: &str = r#"### Open tabs
- 0: (current) [Example Domain] (https://example.com/)

### Page state
- Page URL: https://example.com/
- Page Title: Example Domain
- Page Snapshot:
```yaml
- generic [ref=e2]:
  - heading "Example Domain" [level=1] [ref=e3]
  - paragraph [ref=e4]: This domain is for use in examples.
  - paragraph [ref=e5]:
    - link "Learn more" [ref=e6] [cursor=pointer]:
      - /url: https://iana.org/domains/example
```
"#;

const BLANK_TABS_SNAPSHOT: &str = r#"### Open tabs
- 0: (current) [Time Tracker] (https://example.com/)
- 1: [] (about:blank)
- 2: [] (about:blank)
- 3: [] (about:blank)

### Page state
- Page URL: https://example.com/
- Page Title: Time Tracker
- Page Snapshot:
```yaml
- generic [ref=e1]
```
"#;

#[test]
fn test_meyerhold_summary() {
    let mh = Meyerhold::from_text(SAMPLE_SNAPSHOT);
    let summary = mh.summary();
    assert_eq!(summary.page_url, "https://example.com/");
    assert_eq!(summary.page_title, "Example Domain");
    assert_eq!(summary.tab_count, 1);
    assert_eq!(summary.error_count, 0);
    assert!(summary.element_count > 0);
}

#[test]
fn test_meyerhold_elements_links() {
    let mh = Meyerhold::from_text(SAMPLE_SNAPSHOT);
    let elements = mh.elements(ListType::Links);
    assert_eq!(elements.len(), 1);
    assert_eq!(elements[0].element_type, "link");
    assert_eq!(elements[0].label, "Learn more");
}

#[test]
fn test_meyerhold_elements_headings() {
    let mh = Meyerhold::from_text(SAMPLE_SNAPSHOT);
    let elements = mh.elements(ListType::Headings);
    assert_eq!(elements.len(), 1);
    assert_eq!(elements[0].label, "Example Domain");
}

#[test]
fn test_meyerhold_section_tabs() {
    let mh = Meyerhold::from_text(SAMPLE_SNAPSHOT);
    let result = mh.section(Section::Tabs).unwrap();
    assert!(result.contains("Example Domain"));
    assert!(result.contains("example.com"));
}

#[test]
fn test_meyerhold_section_tree() {
    let mh = Meyerhold::from_text(SAMPLE_SNAPSHOT);
    let result = mh.section(Section::Tree).unwrap();
    assert!(result.contains("[ref=e2]"));
    assert!(result.contains("heading"));
}

#[test]
fn test_meyerhold_blank_tabs() {
    let mh = Meyerhold::from_text(BLANK_TABS_SNAPSHOT);
    let summary = mh.summary();
    assert_eq!(summary.tab_count, 4);
    assert_eq!(summary.blank_tab_count, 3);
    assert_eq!(mh.blank_tab_count(), 3);
}

#[test]
fn test_meyerhold_no_blank_tabs() {
    let mh = Meyerhold::from_text(SAMPLE_SNAPSHOT);
    let summary = mh.summary();
    assert_eq!(summary.tab_count, 1);
    assert_eq!(summary.blank_tab_count, 0);
    assert_eq!(mh.blank_tab_count(), 0);
}

#[test]
fn test_meyerhold_search() {
    let mh = Meyerhold::from_text(SAMPLE_SNAPSHOT);
    let results = mh.search("Example", false).unwrap();
    assert!(!results.is_empty());
}

#[test]
fn test_meyerhold_search_regex() {
    let mh = Meyerhold::from_text(SAMPLE_SNAPSHOT);
    let results = mh.search(r"heading.*Example", true).unwrap();
    assert!(!results.is_empty());
}

#[test]
fn test_meyerhold_search_not_found() {
    let mh = Meyerhold::from_text(SAMPLE_SNAPSHOT);
    let result = mh.search("nonexistent pattern xyz", false);
    assert!(result.is_err());
}

#[test]
fn test_truncate_str_short() {
    assert_eq!(truncate_str("hello", 10), "hello");
}

#[test]
fn test_truncate_str_long() {
    let result = truncate_str("hello world this is long", 10);
    assert!(result.ends_with("..."));
    assert!(result.len() <= 13); // 10 chars + "..."
}

#[test]
fn test_truncate_str_unicode() {
    let result = truncate_str("こんにちは世界", 5);
    assert!(result.ends_with("..."));
}

#[test]
fn test_meyerhold_tree() {
    let mh = Meyerhold::from_text(SAMPLE_SNAPSHOT);
    let tree = mh.tree(3, None);
    assert!(tree.contains("[ref=e2]"));
}
