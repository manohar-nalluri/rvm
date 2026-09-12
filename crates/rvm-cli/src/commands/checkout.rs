use rvm_compiler::Compiler;
use rvm_core::{branch, Workspace};

pub fn execute(name: &str) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let ws = Workspace::discover(&cwd)?;
    branch::checkout(&ws, name)?;
    println!("Switched to branch '{}'", name);

    let config = ws.load_config()?;
    if config.compiler.auto_compile {
        if let Ok(tex_path) = ws.find_tex_file() {
            match Compiler::new(&config.compiler.engine) {
                Ok(compiler) => match compiler.compile(&tex_path) {
                    Ok(result) => {
                        println!("Compiled: {}", result.pdf_path.display());
                    }
                    Err(e) => {
                        eprintln!("Auto-compilation failed: {}", e);
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
