use rvm_core::{commit, SystemAuthenticator, Workspace};

pub fn execute(message: &str) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let ws = Workspace::discover(&cwd)?;
    let branch = ws.current_branch()?;

    // Protected branches require the operator's system password before anything
    // is written. Unprotected branches never prompt.
    let auth = SystemAuthenticator::new();
    let c = commit::create(&ws, message, &auth)?;
    println!("[{}] {} {}", branch, c.hash.short(), message);

    crate::commands::build::run(&ws, true)
}
