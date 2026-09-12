use std::path::Path;

use rvm_core::Workspace;

pub fn execute(branch_name: Option<&str>) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let ws = Workspace::discover(&cwd)?;

    let branch = match branch_name {
        Some(name) => name.to_string(),
        None => ws.current_branch()?,
    };

    let export_dir = ws.root().join("export").join(&branch);
    std::fs::create_dir_all(&export_dir)?;

    let mut exported = Vec::new();

    // Determine the tex/pdf filenames from the workspace's .tex file
    let (tex_name, pdf_name) = match ws.find_tex_file() {
        Ok(tex_path) => {
            let stem = tex_path.file_stem().unwrap().to_string_lossy().to_string();
            (format!("{}.tex", stem), format!("{}.pdf", stem))
        }
        Err(_) => ("resume.tex".to_string(), "resume.pdf".to_string()),
    };

    // Copy .tex snapshot
    let snapshot_path = ws.branches_dir().join(&branch).join("snapshot.tex");
    if snapshot_path.exists() {
        let dest = export_dir.join(&tex_name);
        std::fs::copy(&snapshot_path, &dest)?;
        exported.push(tex_name.clone());
    }

    // Copy .pdf if it exists
    let pdf_path = ws.root().join(&pdf_name);
    if pdf_path.exists() {
        let dest = export_dir.join(&pdf_name);
        std::fs::copy(&pdf_path, &dest)?;
        exported.push(pdf_name);
    }

    // Copy job.json metadata
    let job_path = ws.branches_dir().join(&branch).join("job.json");
    if job_path.exists() {
        let dest = export_dir.join("job.json");
        std::fs::copy(&job_path, &dest)?;
        exported.push("job.json".to_string());
    }

    // Copy commits.json
    let commits_path = ws.branches_dir().join(&branch).join("commits.json");
    if commits_path.exists() {
        let dest = export_dir.join("commits.json");
        std::fs::copy(&commits_path, &dest)?;
        exported.push("commits.json".to_string());
    }

    if exported.is_empty() {
        println!("Nothing to export for branch '{}'.", branch);
    } else {
        println!("Exported branch '{}' to {}:", branch, export_dir.display());
        for file in &exported {
            println!("  {}", file);
        }

        // Try to create a zip if `zip` command is available
        if try_create_zip(&export_dir, &branch, ws.root()) {
            println!(
                "Bundled: {}/{}.zip",
                ws.root().join("export").display(),
                branch
            );
        }
    }

    Ok(())
}

fn try_create_zip(export_dir: &Path, branch: &str, root: &Path) -> bool {
    let zip_path = root.join("export").join(format!("{}.zip", branch));
    let result = std::process::Command::new("zip")
        .arg("-rj")
        .arg(&zip_path)
        .arg(export_dir)
        .output();

    matches!(result, Ok(output) if output.status.success())
}
