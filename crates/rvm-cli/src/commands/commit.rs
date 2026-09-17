use rvm_compiler::Compiler;
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

    compile_and_validate(&ws)
}

/// Auto-compile and validate the current working file, if configured to.
///
/// Shared by `commit` and `restore` so both leave the workspace in the same
/// compiled state. Failures are reported but not fatal, matching `commit`.
pub(crate) fn compile_and_validate(ws: &Workspace) -> anyhow::Result<()> {
    let config = ws.load_config()?;
    if !config.compiler.auto_compile {
        return Ok(());
    }

    let Ok(tex_path) = ws.find_tex_file() else {
        return Ok(());
    };

    match Compiler::new(&config.compiler.engine) {
        Ok(compiler) => match compiler.compile(&tex_path) {
            Ok(result) => {
                println!("Compiled: {}", result.pdf_path.display());
                for w in &result.warnings {
                    println!("  Warning: {}", w);
                }
                let report = rvm_validator::validate(
                    &result.pdf_path,
                    &std::fs::read_to_string(&tex_path)?,
                    &config,
                )?;
                if report.has_errors() {
                    println!("Validation errors:");
                    for d in report.errors() {
                        println!("  [{}] {}", d.code, d.message);
                    }
                } else if !report.diagnostics.is_empty() {
                    println!(
                        "Validation passed with {} warning(s).",
                        report.diagnostics.len()
                    );
                }
            }
            Err(e) => {
                eprintln!("Compilation failed: {}", e);
            }
        },
        Err(e) => {
            eprintln!("Compiler setup failed: {}", e);
        }
    }

    Ok(())
}
