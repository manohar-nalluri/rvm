use rvm_core::{commit, SystemAuthenticator, Workspace};

/// Restore the current branch to the content of a previous commit.
///
/// This adds a new commit rather than rewriting history, so the polluted state
/// remains recoverable and the restore can itself be undone.
pub fn execute(reference: &str) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let ws = Workspace::discover(&cwd)?;
    let branch = ws.current_branch()?;

    // Main is often protected, and restoring it is a mutation like any other,
    // so this authenticates the same way `commit` does.
    let auth = SystemAuthenticator::new();
    let (target, new_commit) = commit::restore(&ws, &branch, reference, &auth)?;

    println!(
        "Restored '{}' to {} ({})",
        branch,
        target.hash.short(),
        target.message.lines().next().unwrap_or("").trim()
    );
    println!("[{}] {} {}", branch, new_commit.hash.short(), new_commit.message);
    println!();
    println!(
        "History was not rewritten: the previous state is still recoverable with \
         `rvm restore {}`.",
        new_commit.parent.as_ref().map(|p| p.short()).unwrap_or("HEAD")
    );

    crate::commands::commit::compile_and_validate(&ws)
}
