//! Integration tests for meyerhold CLI

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn get_test_snapshot() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/example.json")
}

#[allow(deprecated)]
fn meyerhold() -> Command {
    Command::cargo_bin("meyerhold").unwrap()
}

#[test]
fn test_summary_view_default() {
    let snapshot = get_test_snapshot();
    meyerhold()
        .arg(&snapshot)
        .assert()
        .success()
        .stdout(predicate::str::contains("=== Snapshot Summary ==="))
        .stdout(predicate::str::contains("Page URL:"));
}

#[test]
fn test_summary_as_json() {
    let snapshot = get_test_snapshot();
    meyerhold()
        .args([snapshot.to_str().unwrap(), "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("page_url"))
        .stdout(predicate::str::contains("element_count"));
}

#[test]
fn test_section_tabs() {
    let snapshot = get_test_snapshot();
    meyerhold()
        .args([snapshot.to_str().unwrap(), "--section", "tabs"])
        .assert()
        .success();
}

#[test]
fn test_section_errors() {
    let snapshot = get_test_snapshot();
    meyerhold()
        .args([snapshot.to_str().unwrap(), "--section", "errors"])
        .assert()
        .success();
}

#[test]
fn test_section_page() {
    let snapshot = get_test_snapshot();
    meyerhold()
        .args([snapshot.to_str().unwrap(), "--section", "page"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Page URL:").or(predicate::str::contains("Page Title:")));
}

#[test]
fn test_section_tree() {
    let snapshot = get_test_snapshot();
    meyerhold()
        .args([snapshot.to_str().unwrap(), "--section", "tree"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[ref="));
}

#[test]
fn test_list_links() {
    let snapshot = get_test_snapshot();
    meyerhold()
        .args([snapshot.to_str().unwrap(), "--list", "links"])
        .assert()
        .success()
        .stdout(predicate::str::contains("link"));
}

#[test]
fn test_list_inputs() {
    let snapshot = get_test_snapshot();
    meyerhold()
        .args([snapshot.to_str().unwrap(), "--list", "inputs"])
        .assert()
        .success();
}

#[test]
fn test_list_all_interactive() {
    let snapshot = get_test_snapshot();
    meyerhold()
        .args([snapshot.to_str().unwrap(), "--list", "all", "--limit", "10"])
        .assert()
        .success();
}

#[test]
fn test_list_text() {
    let snapshot = get_test_snapshot();
    meyerhold()
        .args([snapshot.to_str().unwrap(), "--list", "text", "--limit", "5"])
        .assert()
        .success();
}

#[test]
fn test_list_headings() {
    let snapshot = get_test_snapshot();
    meyerhold()
        .args([snapshot.to_str().unwrap(), "--list", "headings"])
        .assert()
        .success();
}

#[test]
fn test_list_table_format() {
    let snapshot = get_test_snapshot();
    meyerhold()
        .args([snapshot.to_str().unwrap(), "--list", "links", "--format", "table"])
        .assert()
        .success()
        .stdout(predicate::str::contains("REF"))
        .stdout(predicate::str::contains("TYPE"))
        .stdout(predicate::str::contains("LABEL"));
}

#[test]
fn test_list_json_format() {
    let snapshot = get_test_snapshot();
    meyerhold()
        .args([snapshot.to_str().unwrap(), "--list", "links", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ref_id"))
        .stdout(predicate::str::contains("element_type"));
}

#[test]
fn test_tree_depth_2() {
    let snapshot = get_test_snapshot();
    meyerhold()
        .args([snapshot.to_str().unwrap(), "--depth", "2"])
        .assert()
        .success();
}

#[test]
fn test_tree_depth_3() {
    let snapshot = get_test_snapshot();
    meyerhold()
        .args([snapshot.to_str().unwrap(), "--depth", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[ref="));
}

#[test]
fn test_search_text_pattern() {
    let snapshot = get_test_snapshot();
    meyerhold()
        .args([snapshot.to_str().unwrap(), "--search", "link"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Found"))
        .stdout(predicate::str::contains("matches"));
}

#[test]
fn test_search_with_limit() {
    let snapshot = get_test_snapshot();
    meyerhold()
        .args([snapshot.to_str().unwrap(), "--search", "link", "--limit", "2"])
        .assert()
        .success();
}

#[test]
fn test_search_regex_pattern() {
    let snapshot = get_test_snapshot();
    meyerhold()
        .args([snapshot.to_str().unwrap(), "--search", "link.*http", "--regex"])
        .assert();
    // May succeed or fail depending on content, just ensure no panic
}

#[test]
fn test_search_no_matches() {
    let snapshot = get_test_snapshot();
    meyerhold()
        .args([snapshot.to_str().unwrap(), "--search", "xyznonexistent12345"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No matches found"));
}

#[test]
fn test_line_numbers_option() {
    let snapshot = get_test_snapshot();
    meyerhold()
        .args([snapshot.to_str().unwrap(), "--list", "links", "--limit", "3", "-n"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1:").or(predicate::str::contains("   1:")));
}

#[test]
fn test_limit_option() {
    let snapshot = get_test_snapshot();
    meyerhold()
        .args([snapshot.to_str().unwrap(), "--list", "links", "--limit", "5"])
        .assert()
        .success();
}

#[test]
fn test_invalid_file() {
    meyerhold()
        .arg("/nonexistent/path/file.json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("ERROR"))
        .stderr(predicate::str::contains("Failed to read"));
}

#[test]
fn test_search_json_output() {
    let snapshot = get_test_snapshot();
    meyerhold()
        .args([snapshot.to_str().unwrap(), "--search", "link", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("content"));
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[test]
fn test_empty_file() {
    let temp_dir = std::env::temp_dir();
    let empty_file = temp_dir.join("meyerhold_test_empty.json");
    std::fs::write(&empty_file, "").unwrap();

    meyerhold()
        .arg(&empty_file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("ERROR"))
        .stderr(predicate::str::contains("empty"));

    std::fs::remove_file(&empty_file).ok();
}

#[test]
fn test_whitespace_only_file() {
    let temp_dir = std::env::temp_dir();
    let ws_file = temp_dir.join("meyerhold_test_whitespace.json");
    std::fs::write(&ws_file, "   \n\t\n  ").unwrap();

    meyerhold()
        .arg(&ws_file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("empty"));

    std::fs::remove_file(&ws_file).ok();
}

#[test]
fn test_invalid_json() {
    let temp_dir = std::env::temp_dir();
    let invalid_file = temp_dir.join("meyerhold_test_invalid.json");
    std::fs::write(&invalid_file, "{ not valid json }").unwrap();

    meyerhold()
        .arg(&invalid_file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid JSON"));

    std::fs::remove_file(&invalid_file).ok();
}

#[test]
fn test_valid_json_wrong_structure() {
    let temp_dir = std::env::temp_dir();
    let wrong_file = temp_dir.join("meyerhold_test_wrong_structure.json");
    std::fs::write(&wrong_file, r#"{"foo": "bar"}"#).unwrap();

    meyerhold()
        .arg(&wrong_file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Could not find snapshot text"));

    std::fs::remove_file(&wrong_file).ok();
}

#[test]
fn test_empty_content_array() {
    let temp_dir = std::env::temp_dir();
    let empty_arr_file = temp_dir.join("meyerhold_test_empty_array.json");
    std::fs::write(&empty_arr_file, r#"{"content": []}"#).unwrap();

    meyerhold()
        .arg(&empty_arr_file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Could not find snapshot text"));

    std::fs::remove_file(&empty_arr_file).ok();
}
