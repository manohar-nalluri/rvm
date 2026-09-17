use rvm_core::{branch, commit, diff, Workspace};

pub fn execute(a: Option<&str>, b: Option<&str>) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let ws = Workspace::discover(&cwd)?;

    let reference_a = a
        .map(|s| s.to_string())
        .unwrap_or_else(|| ws.current_branch().unwrap_or_default());

    let content_a = load_content(&ws, &reference_a)?;

    let content_b = match b {
        Some(reference) => load_content(&ws, reference)?,
        None => {
            // Diff the first reference against the working file.
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

/// Load content for a branch name or a commit reference.
///
/// This previously understood only branch names even though the command
/// documents itself as accepting commits, so passing a hash silently compared
/// *empty* content against the file and printed the entire resume as added
/// lines. An unresolvable reference is now a hard error rather than a wrong
/// answer.
fn load_content(ws: &Workspace, reference: &str) -> anyhow::Result<String> {
    if branch::exists(ws, reference) {
        let snapshot = ws.branches_dir().join(reference).join("snapshot.tex");
        return Ok(if snapshot.exists() {
            std::fs::read_to_string(snapshot)?
        } else {
            String::new()
        });
    }

    let current = ws.current_branch()?;
    match commit::resolve(ws, &current, reference) {
        Ok(c) => Ok(c.snapshot_tex),
        Err(_) => anyhow::bail!(
            "'{}' is not a branch, nor a commit on branch '{}'",
            reference,
            current
        ),
    }
}
