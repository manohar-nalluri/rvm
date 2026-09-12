use rvm_core::{diff, Workspace};

pub fn execute(a: Option<&str>, b: Option<&str>) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let ws = Workspace::discover(&cwd)?;

    let branch_a = a
        .map(|s| s.to_string())
        .unwrap_or_else(|| ws.current_branch().unwrap_or_default());

    let content_a = load_branch_content(&ws, &branch_a)?;

    let content_b = match b {
        Some(name) => load_branch_content(&ws, name)?,
        None => {
            // Diff current working file against latest commit
            match ws.find_tex_file() {
                Ok(tex_path) => std::fs::read_to_string(tex_path)?,
                Err(_) => String::new(),
            }
        }
    };

    let output = diff::format_unified(&content_a, &content_b, 3);

    if output.is_empty() {
        println!("No differences found.");
    } else {
        print!("{}", output);
    }

    Ok(())
}

fn load_branch_content(ws: &Workspace, name: &str) -> anyhow::Result<String> {
    let snapshot = ws.branches_dir().join(name).join("snapshot.tex");
    if snapshot.exists() {
        Ok(std::fs::read_to_string(snapshot)?)
    } else {
        Ok(String::new())
    }
}
