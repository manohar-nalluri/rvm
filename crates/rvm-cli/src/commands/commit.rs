use rvm_compiler::Compiler;
use rvm_core::{commit, Workspace};

pub fn execute(message: &str) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let ws = Workspace::discover(&cwd)?;
    let branch = ws.current_branch()?;
    let c = commit::create(&ws, message)?;
    println!("[{}] {} {}", branch, c.hash.short(), message);

    let config = ws.load_config()?;
    if config.compiler.auto_compile {
        if let Ok(tex_path) = ws.find_tex_file() {
            match Compiler::new(&config.compiler.engine) {
                Ok(compiler) => match compiler.compile(&tex_path) {
                    Ok(result) => {
                        println!("Compiled: {}", result.pdf_path.display());
                        for w in &result.warnings {
                            println!("  Warning: {}", w);
                        }
                        // Run validation
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
        }
    }

    Ok(())
}
