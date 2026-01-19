//! Shared utility functions for parsing and formatting.

use crate::summary::ContentItem;

/// Calculate depth from line indentation (2 spaces per level).
pub fn calculate_depth(line: &str) -> usize {
    let leading_spaces = line.len() - line.trim_start().len();
    leading_spaces / 2
}

/// Truncate label at character boundary (UTF-8 safe).
pub fn truncate_label(s: &str, max_chars: usize) -> String {
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

/// Extract quoted content from a line (e.g., `heading "Title"` -> `Title`).
pub fn extract_quoted_content(line: &str) -> Option<String> {
    let start = line.find('"')?;
    let end = line.rfind('"')?;
    if start < end {
        Some(line[start + 1..end].to_string())
    } else {
        None
    }
}

/// Tracker for visible ancestor depth with O(1) depth calculation.
///
/// Maintains a stack of (dom_depth, is_content_item) and tracks
/// visible_depth incrementally instead of counting on each access.
#[derive(Debug, Default)]
pub struct AncestorTracker {
    /// Stack of (dom_depth, is_content_item)
    stack: Vec<(usize, bool)>,
    /// Current count of visible (content item) ancestors
    visible_depth: usize,
}

impl AncestorTracker {
    /// Create a new empty tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Pop ancestors that are not parents of the current depth.
    /// Call this before processing each line.
    pub fn pop_non_ancestors(&mut self, current_depth: usize) {
        while let Some(&(d, is_content)) = self.stack.last() {
            if d >= current_depth {
                if is_content {
                    self.visible_depth = self.visible_depth.saturating_sub(1);
                }
                self.stack.pop();
            } else {
                break;
            }
        }
    }

    /// Get current visible depth (O(1)).
    pub fn visible_depth(&self) -> usize {
        self.visible_depth
    }

    /// Push a new item onto the stack.
    pub fn push(&mut self, dom_depth: usize, is_content_item: bool) {
        if is_content_item {
            self.visible_depth += 1;
        }
        self.stack.push((dom_depth, is_content_item));
    }
}

/// Set depth on a ContentItem.
pub fn set_item_depth(item: &mut ContentItem, depth: usize) {
    match item {
        ContentItem::Heading { depth: d, .. } => *d = depth,
        ContentItem::Text { depth: d, .. } => *d = depth,
        ContentItem::Button { depth: d, .. } => *d = depth,
        ContentItem::Link { depth: d, .. } => *d = depth,
        ContentItem::Input { depth: d, .. } => *d = depth,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_depth() {
        assert_eq!(calculate_depth("root"), 0);
        assert_eq!(calculate_depth("  level1"), 1);
        assert_eq!(calculate_depth("    level2"), 2);
        assert_eq!(calculate_depth("      level3"), 3);
    }

    #[test]
    fn test_truncate_label_short() {
        assert_eq!(truncate_label("short", 10), "short");
    }

    #[test]
    fn test_truncate_label_long() {
        assert_eq!(truncate_label("this is a very long string", 10), "this is...");
    }

    #[test]
    fn test_truncate_label_unicode() {
        assert_eq!(truncate_label("日本語テスト", 5), "日本...");
    }

    #[test]
    fn test_extract_quoted_content() {
        assert_eq!(
            extract_quoted_content(r#"heading "Hello World""#),
            Some("Hello World".to_string())
        );
        assert_eq!(extract_quoted_content("no quotes"), None);
        assert_eq!(extract_quoted_content(r#"single "quote"#), None);
    }

    #[test]
    fn test_ancestor_tracker_basic() {
        let mut tracker = AncestorTracker::new();

        // Root level content item
        tracker.pop_non_ancestors(0);
        assert_eq!(tracker.visible_depth(), 0);
        tracker.push(0, true);

        // Child level content item
        tracker.pop_non_ancestors(1);
        assert_eq!(tracker.visible_depth(), 1);
        tracker.push(1, true);

        // Grandchild level content item
        tracker.pop_non_ancestors(2);
        assert_eq!(tracker.visible_depth(), 2);
        tracker.push(2, true);

        // Back to sibling of child (depth 1)
        tracker.pop_non_ancestors(1);
        assert_eq!(tracker.visible_depth(), 1);
    }

    #[test]
    fn test_ancestor_tracker_non_content() {
        let mut tracker = AncestorTracker::new();

        // Root level non-content
        tracker.pop_non_ancestors(0);
        tracker.push(0, false);
        assert_eq!(tracker.visible_depth(), 0);

        // Child level content item
        tracker.pop_non_ancestors(1);
        assert_eq!(tracker.visible_depth(), 0); // Parent is not content
        tracker.push(1, true);

        // Grandchild level content item
        tracker.pop_non_ancestors(2);
        assert_eq!(tracker.visible_depth(), 1); // Only one visible ancestor
    }
}
