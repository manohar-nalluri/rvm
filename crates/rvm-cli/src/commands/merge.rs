use std::collections::HashSet;

use rvm_core::{commit, guard, merge, SystemAuthenticator, Workspace};
use rvm_types::CommitHash;

pub fn execute(branch_name: &str) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let ws = Workspace::discover(&cwd)?;

    let current = ws.current_branch()?;

    // Merging rewrites the current branch's content, so it is a mutation of the
    // destination branch and is gated the same way a commit is. Without this an
    // agent could land its work on a protected branch by merging instead.
    let auth = SystemAuthenticator::new();
    guard::ensure_mutable(&ws, &current, &auth)?;

    // Load the content from both branches
    let ours_path = ws.branches_dir().join(&current).join("snapshot.tex");
    let theirs_path = ws.branches_dir().join(branch_name).join("snapshot.tex");

    let ours = if ours_path.exists() {
        std::fs::read_to_string(&ours_path)?
    } else {
        String::new()
    };

    let theirs = if theirs_path.exists() {
        std::fs::read_to_string(&theirs_path)?
    } else {
        anyhow::bail!("Branch '{}' has no snapshot to merge.", branch_name);
    };

    // Find common ancestor by comparing commit histories
    let ancestor = find_common_ancestor_content(&ws, &current, branch_name)?;

    let result = merge::three_way(&ancestor, &ours, &theirs);

    // Write the merged content to the working file
    let tex_path = ws.find_tex_file().unwrap_or_else(|_| ws.root().join("resume.tex"));
    std::fs::write(&tex_path, &result.content)?;

    if result.has_conflicts {
        let tex_name = tex_path.file_name().unwrap().to_string_lossy();
        println!(
            "Merged '{}' into '{}' with {} conflict(s).",
            branch_name, current, result.conflict_count
        );
        println!("Resolve conflicts in {}, then run `rvm commit`.", tex_name);
    } else {
        println!("Merged '{}' into '{}' cleanly.", branch_name, current);
    }

    Ok(())
}

/// Find the content at the common ancestor commit between two branches.
fn find_common_ancestor_content(
    ws: &Workspace,
    branch_a: &str,
    branch_b: &str,
) -> anyhow::Result<String> {
    let history_a = commit::load_history(ws, branch_a)?;
    let history_b = commit::load_history(ws, branch_b)?;

    // Build a set of commit hashes from branch A
    let hashes_a: HashSet<&CommitHash> = history_a.iter().map(|c| &c.hash).collect();

    // Walk branch B's history backwards to find the most recent common commit
    for commit_b in history_b.iter().rev() {
        if hashes_a.contains(&commit_b.hash) {
            return Ok(commit_b.snapshot_tex.clone());
        }
    }

    // No common ancestor found - use empty string (full diff)
    Ok(String::new())
}
