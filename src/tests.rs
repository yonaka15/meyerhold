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

#[test]
fn test_extract_snapshot_text_valid() {
    let json: Value = serde_json::json!({
        "content": [{"text": "hello world", "type": "text"}]
    });
    let result = extract_snapshot_text(&json);
    assert_eq!(result, Ok("hello world".to_string()));
}

#[test]
fn test_extract_snapshot_text_missing_content() {
    let json: Value = serde_json::json!({});
    let result = extract_snapshot_text(&json);
    assert!(result.is_err());
}

#[test]
fn test_extract_snapshot_text_empty_array() {
    let json: Value = serde_json::json!({"content": []});
    let result = extract_snapshot_text(&json);
    assert!(result.is_err());
}

#[test]
fn test_parse_summary() {
    let summary = parse_summary(SAMPLE_SNAPSHOT);
    assert_eq!(summary.page_url, "https://example.com/");
    assert_eq!(summary.page_title, "Example Domain");
    assert_eq!(summary.tab_count, 1);
    assert_eq!(summary.error_count, 0);
    assert!(summary.element_count > 0);
}

#[test]
fn test_extract_section_tabs() {
    let result = extract_section(SAMPLE_SNAPSHOT, SECTION_TABS, SECTION_END);
    assert!(result.contains("Example Domain"));
    assert!(result.contains("example.com"));
}

#[test]
fn test_extract_section_tree() {
    let result = extract_section(SAMPLE_SNAPSHOT, SECTION_TREE_START, SECTION_TREE_END);
    assert!(result.contains("[ref=e2]"));
    assert!(result.contains("heading"));
}

#[test]
fn test_extract_elements_links() {
    let elements = extract_elements(SAMPLE_SNAPSHOT, &ListType::Links);
    assert_eq!(elements.len(), 1);
    assert_eq!(elements[0].element_type, "link");
    assert_eq!(elements[0].label, "Learn more");
}

#[test]
fn test_extract_elements_headings() {
    let elements = extract_elements(SAMPLE_SNAPSHOT, &ListType::Headings);
    assert_eq!(elements.len(), 1);
    assert_eq!(elements[0].label, "Example Domain");
}

#[test]
fn test_extract_label() {
    assert_eq!(extract_label(r#"button "Click me" [ref=e1]"#), "Click me");
    assert_eq!(extract_label(r#"link "Home" [ref=e2]"#), "Home");
    assert_eq!(extract_label(r#"heading "Title" [level=1]"#), "Title");
}

#[test]
fn test_extract_label_no_quotes() {
    assert_eq!(extract_label("button [ref=e1]"), "");
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
fn test_ref_regex() {
    assert!(REF_REGEX.is_match("[ref=e123]"));
    assert!(REF_REGEX.is_match("[ref=abc-def]"));
    assert!(!REF_REGEX.is_match("[ref=]"));
}

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
fn test_parse_summary_blank_tabs() {
    let summary = parse_summary(BLANK_TABS_SNAPSHOT);
    assert_eq!(summary.tab_count, 4);
    assert_eq!(summary.blank_tab_count, 3);
}

#[test]
fn test_parse_summary_no_blank_tabs() {
    let summary = parse_summary(SAMPLE_SNAPSHOT);
    assert_eq!(summary.tab_count, 1);
    assert_eq!(summary.blank_tab_count, 0);
}
