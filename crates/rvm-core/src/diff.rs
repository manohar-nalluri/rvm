use similar::{ChangeTag, TextDiff};

/// Represents a single change in a diff.
#[derive(Debug, Clone)]
pub struct DiffLine {
    pub tag: DiffTag,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffTag {
    Equal,
    Insert,
    Delete,
}

/// Compute a line-level diff between two strings.
pub fn compute(old: &str, new: &str) -> Vec<DiffLine> {
    let diff = TextDiff::from_lines(old, new);
    diff.iter_all_changes()
        .map(|change| {
            let tag = match change.tag() {
                ChangeTag::Equal => DiffTag::Equal,
                ChangeTag::Insert => DiffTag::Insert,
                ChangeTag::Delete => DiffTag::Delete,
            };
            DiffLine {
                tag,
                content: change.value().to_string(),
            }
        })
        .collect()
}

/// Format a diff for terminal display.
pub fn format_unified(old: &str, new: &str, context_lines: usize) -> String {
    let diff = TextDiff::from_lines(old, new);
    diff.unified_diff()
        .context_radius(context_lines)
        .to_string()
}

/// Check if two strings are identical (no diff).
pub fn is_identical(old: &str, new: &str) -> bool {
    old == new
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_strings() {
        let lines = compute("hello\n", "hello\n");
        assert!(lines.iter().all(|l| l.tag == DiffTag::Equal));
        assert!(is_identical("hello", "hello"));
    }

    #[test]
    fn test_insertion() {
        let lines = compute("line1\n", "line1\nline2\n");
        assert!(lines.iter().any(|l| l.tag == DiffTag::Insert));
    }

    #[test]
    fn test_deletion() {
        let lines = compute("line1\nline2\n", "line1\n");
        assert!(lines.iter().any(|l| l.tag == DiffTag::Delete));
    }

    #[test]
    fn test_format_unified_empty_diff() {
        let output = format_unified("same\n", "same\n", 3);
        assert!(output.is_empty());
    }

    #[test]
    fn test_format_unified_with_changes() {
        let output = format_unified("old\n", "new\n", 3);
        assert!(output.contains("-old"));
        assert!(output.contains("+new"));
    }

    #[test]
    fn test_not_identical() {
        assert!(!is_identical("a", "b"));
    }
}
