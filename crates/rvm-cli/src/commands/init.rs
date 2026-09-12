use rvm_core::Workspace;

pub fn execute() -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    Workspace::init(&cwd)?;
    println!("Initialized RVM workspace in {}", cwd.display());
    println!("Created branch 'main' as the base resume.");
    println!("\nNext steps:");
    println!("  1. Place your .tex file in this directory (e.g. resume.tex, yourname.tex)");
    println!("  2. Run `rvm commit -m \"Initial resume\"` to save your first version");
    println!("  3. Run `rvm branch <company-role>` to create tailored versions");
    println!("\nNote: Only one .tex file per workspace. The filename can be anything you like.");
    Ok(())
}
