/// Result of a three-way merge.
#[derive(Debug)]
pub struct MergeResult {
    pub content: String,
    pub has_conflicts: bool,
    pub conflict_count: usize,
}

/// Perform a three-way merge.
///
/// Given a common ancestor, the current branch content, and the incoming branch
/// content, produce merged output with conflict markers where needed.
pub fn three_way(ancestor: &str, ours: &str, theirs: &str) -> MergeResult {
    // If either side is identical to the ancestor, take the other side
    if ours == ancestor {
        return MergeResult {
            content: theirs.to_string(),
            has_conflicts: false,
            conflict_count: 0,
        };
    }
    if theirs == ancestor {
        return MergeResult {
            content: ours.to_string(),
            has_conflicts: false,
            conflict_count: 0,
        };
    }

    // Both sides made the same change
    if ours == theirs {
        return MergeResult {
            content: ours.to_string(),
            has_conflicts: false,
            conflict_count: 0,
        };
    }

    // Both sides changed differently - mark as conflict.
    // TODO: Implement proper three-way merge algorithm with non-overlapping region detection.
    let mut result = String::new();
    let conflict_count = 1;

    result.push_str("<<<<<<< ours\n");
    result.push_str(ours);
    if !ours.ends_with('\n') {
        result.push('\n');
    }
    result.push_str("=======\n");
    result.push_str(theirs);
    if !theirs.ends_with('\n') {
        result.push('\n');
    }
    result.push_str(">>>>>>> theirs\n");

    MergeResult {
        content: result,
        has_conflicts: conflict_count > 0,
        conflict_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ours_unchanged_takes_theirs() {
        let result = three_way("base", "base", "modified");
        assert!(!result.has_conflicts);
        assert_eq!(result.content, "modified");
    }

    #[test]
    fn test_theirs_unchanged_takes_ours() {
        let result = three_way("base", "modified", "base");
        assert!(!result.has_conflicts);
        assert_eq!(result.content, "modified");
    }

    #[test]
    fn test_same_change_no_conflict() {
        let result = three_way("base", "same", "same");
        assert!(!result.has_conflicts);
        assert_eq!(result.content, "same");
    }

    #[test]
    fn test_different_changes_conflict() {
        let result = three_way("base", "ours_change", "theirs_change");
        assert!(result.has_conflicts);
        assert_eq!(result.conflict_count, 1);
        assert!(result.content.contains("<<<<<<< ours"));
        assert!(result.content.contains("======="));
        assert!(result.content.contains(">>>>>>> theirs"));
    }
}
