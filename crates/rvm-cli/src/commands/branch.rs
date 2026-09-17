use rvm_core::{branch, config, guard, SystemAuthenticator, Workspace};

pub fn execute(
    name: Option<&str>,
    list: bool,
    archived: bool,
    protect: Option<&str>,
    unprotect: Option<&str>,
) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let ws = Workspace::discover(&cwd)?;
    let auth = SystemAuthenticator::new();

    // --- Protection changes -------------------------------------------------
    //
    // Both directions are gated. Unprotecting obviously is: it is the operation
    // that would let an agent write to a protected branch. Protecting is gated
    // too, because otherwise an agent could protect every branch and lock the
    // operator out of their own work.
    if let Some(target) = protect {
        require_branch(&ws, target)?;
        guard::ensure_mutable(&ws, target, &auth)?;
        if config::is_protected(&ws, target)? {
            println!("Branch '{}' is already protected.", target);
        } else {
            config::set_protected(&ws, target, true)?;
            println!("Protected branch '{}'.", target);
            println!("Modifying it now requires your system password.");
        }
        return Ok(());
    }

    if let Some(target) = unprotect {
        require_branch(&ws, target)?;
        guard::ensure_mutable(&ws, target, &auth)?;
        if !config::is_protected(&ws, target)? {
            println!("Branch '{}' is not protected.", target);
        } else {
            config::set_protected(&ws, target, false)?;
            println!("Removed protection from branch '{}'.", target);
        }
        return Ok(());
    }

    // --- Listing ------------------------------------------------------------
    let protected = config::list_protected(&ws)?;

    if list || (name.is_none() && !archived) {
        // List active branches
        let current = ws.current_branch()?;
        let branches = branch::list(&ws)?;
        if branches.is_empty() {
            println!("No branches found.");
        } else {
            for b in &branches {
                let prefix = if *b == current { "* " } else { "  " };
                println!("{}{}{}", prefix, b, lock_marker(&protected, b));
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
                println!("  {}{}", b, lock_marker(&protected, b));
            }
        }
        return Ok(());
    }

    // --- Create a new branch ------------------------------------------------
    if let Some(name) = name {
        let current = ws.current_branch()?;
        branch::create(&ws, name)?;
        let mut message = format!("Created branch '{}' from '{}'", name, current);
        if config::is_protected(&ws, name)? {
            message.push_str(" (protected)");
        }
        println!("{}", message);
    }

    Ok(())
}

fn require_branch(ws: &Workspace, name: &str) -> anyhow::Result<()> {
    if !branch::exists(ws, name) {
        anyhow::bail!("Branch '{}' not found", name);
    }
    Ok(())
}

fn lock_marker(protected: &[String], branch: &str) -> &'static str {
    if protected.iter().any(|b| b == branch) {
        " [protected]"
    } else {
        ""
    }
}
