use rvm_core::{commit, Workspace};

pub fn execute() -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let ws = Workspace::discover(&cwd)?;
    let branch = ws.current_branch()?;
    let history = commit::load_history(&ws, &branch)?;

    if history.is_empty() {
        println!("No commits on branch '{}'.", branch);
        return Ok(());
    }

    println!("Commit log for branch '{}':\n", branch);

    for c in history.iter().rev() {
        let time = c.timestamp.format("%Y-%m-%d %H:%M:%S");
        println!(
            "\x1b[33m{}\x1b[0m {}",
            c.hash.short(),
            c.message
        );
        println!("  Date: {}\n", time);
    }

    Ok(())
}
