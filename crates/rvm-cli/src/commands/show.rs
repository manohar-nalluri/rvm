use rvm_core::{branch, commit, Workspace};

/// Print the resume content stored in a previous commit.
///
/// The `.tex` content goes to **stdout** and the metadata to **stderr**, so the
/// output can be redirected straight into a file:
///
/// ```text
/// rvm show 58676505 > old.tex
/// ```
pub fn execute(reference: &str) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let ws = Workspace::discover(&cwd)?;

    // A branch name means "that branch's tip"; anything else is resolved as a
    // commit on the current branch. This mirrors how `diff` accepts either.
    let (branch_name, c) = if branch::exists(&ws, reference) {
        let name = reference.to_string();
        let c = commit::resolve(&ws, &name, "HEAD")?;
        (name, c)
    } else {
        let name = ws.current_branch()?;
        let c = commit::resolve(&ws, &name, reference)?;
        (name, c)
    };

    eprintln!(
        "commit {} on branch '{}'",
        c.hash.short(),
        branch_name
    );
    eprintln!("Date:    {}", c.timestamp);
    eprintln!("Message: {}", c.message);
    eprintln!("---");

    print!("{}", c.snapshot_tex);

    Ok(())
}
