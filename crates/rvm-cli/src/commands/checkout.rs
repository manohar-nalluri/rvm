use rvm_core::{branch, Workspace};

pub fn execute(name: &str) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let ws = Workspace::discover(&cwd)?;
    branch::checkout(&ws, name)?;
    println!("Switched to branch '{}'", name);

    // The branch's `.tex` is now the working file, so the PDF and DOCX are
    // rebuilt from it exactly as they are after a commit.
    crate::commands::build::run(&ws, false)
}
