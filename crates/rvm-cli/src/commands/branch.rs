use rvm_core::{branch, Workspace};

pub fn execute(name: Option<&str>, list: bool, archived: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let ws = Workspace::discover(&cwd)?;

    if list || (name.is_none() && !archived) {
        // List active branches
        let current = ws.current_branch()?;
        let branches = branch::list(&ws)?;
        if branches.is_empty() {
            println!("No branches found.");
        } else {
            for b in &branches {
                let prefix = if *b == current { "* " } else { "  " };
                println!("{}{}", prefix, b);
            }
        }
        return Ok(());
    }

    if archived {
        let archived_branches = branch::list_archived(&ws)?;
        if archived_branches.is_empty() {
            println!("No archived branches.");
        } else {
            println!("Archived branches:");
            for b in &archived_branches {
                println!("  {}", b);
            }
        }
        return Ok(());
    }

    // Create a new branch
    if let Some(name) = name {
        let current = ws.current_branch()?;
        branch::create(&ws, name)?;
        println!("Created branch '{}' from '{}'", name, current);
    }

    Ok(())
}
