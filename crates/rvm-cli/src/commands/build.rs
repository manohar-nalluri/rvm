//! Keep a branch's compiled artifacts in step with its `.tex`.
//!
//! `commit`, `restore` and `checkout` all end with the same job: compile the
//! file that is now in the working tree, refresh the `.docx` beside it, and —
//! for the mutating commands — validate the result. It lives here once so the
//! three cannot drift apart. `checkout` previously carried its own copy of the
//! compile block, which is exactly how the `.docx` would have come to be
//! missing after a branch switch.

use std::path::Path;

use rvm_compiler::{docx, Compiler};
use rvm_core::Workspace;
use rvm_types::RvmConfig;

/// Compile the working `.tex` and refresh every artifact derived from it.
///
/// `validate` is true for `commit`/`restore` and false for `checkout`: a
/// checkout restores content that was already validated when it was committed,
/// so re-reporting it on every branch switch is noise.
///
/// Failures stay non-fatal, matching the behaviour `commit` has always had: a
/// broken `.tex` is reported and the commit it followed still stands.
pub(crate) fn run(ws: &Workspace, validate: bool) -> anyhow::Result<()> {
    let config = ws.load_config()?;
    if !config.compiler.auto_compile {
        return Ok(());
    }

    let Ok(tex_path) = ws.find_tex_file() else {
        return Ok(());
    };

    let compiler = match Compiler::new(&config.compiler.engine) {
        Ok(compiler) => compiler,
        Err(e) => {
            eprintln!("Compiler setup failed: {}", e);
            return Ok(());
        }
    };

    let result = match compiler.compile(&tex_path) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Compilation failed: {}", e);
            return Ok(());
        }
    };

    println!("Compiled: {}", result.pdf_path.display());
    for w in &result.warnings {
        println!("  Warning: {}", w);
    }

    emit_docx(&tex_path, &config);

    if validate {
        emit_validation(&result.pdf_path, &tex_path, &config)?;
    }

    Ok(())
}

/// Rebuild the `.docx` from the same `.tex` the PDF came from.
///
/// Reported, never fatal, and only attempted once the PDF exists. A commit
/// whose PDF compiled has succeeded; a missing pandoc must not turn that into a
/// failed commit.
fn emit_docx(tex_path: &Path, config: &RvmConfig) {
    if !config.compiler.docx {
        return;
    }

    match docx::convert(tex_path) {
        Ok(result) => {
            println!("DOCX: {}", result.docx_path.display());
            for w in &result.warnings {
                println!("  Warning: {}", w);
            }
        }
        Err(e) => {
            eprintln!("DOCX skipped: {}", e);
        }
    }
}

fn emit_validation(pdf_path: &Path, tex_path: &Path, config: &RvmConfig) -> anyhow::Result<()> {
    let source = std::fs::read_to_string(tex_path)?;
    let report = rvm_validator::validate(pdf_path, &source, config)?;

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

    Ok(())
}
