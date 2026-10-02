//! Convert a resume's `.tex` source into a `.docx`.
//!
//! RVM shells out to `pandoc` for this, for the same reason it shells out to
//! `tectonic` for the PDF: reading LaTeX is a large, well-tested job, and the
//! `.tex` is the only input that still carries the document's *structure*.
//!
//! Converting the compiled *PDF* instead was the obvious alternative and was
//! rejected. A PDF has no headings, bold runs or list items — only glyphs at
//! coordinates — so anything rebuilt from it is positioned text. That is
//! precisely the shape an applicant tracking system fails to read, which is
//! the failure mode `docs/ATS-GUIDE.md` exists to avoid.
//!
//! Nothing here is fatal to a commit: the PDF is the artifact that gates
//! everything, and a machine without pandoc must still be able to commit.

use std::path::{Path, PathBuf};
use std::process::Command;

use rvm_types::{RvmError, RvmResult};

/// The converter RVM drives.
pub const DEFAULT_BINARY: &str = "pandoc";

/// Environment variable that overrides the converter binary.
///
/// Exists for the same reason `RVM_AI_CLI` does: a test harness can point RVM
/// at a stub without depending on what happens to be installed on the machine.
pub const BINARY_ENV: &str = "RVM_PANDOC";

#[derive(Debug)]
pub struct DocxResult {
    pub docx_path: PathBuf,
    /// Non-fatal complaints from pandoc, e.g. a macro it could not interpret.
    pub warnings: Vec<String>,
}

/// Convert `tex_path` to a `.docx` sitting beside it.
///
/// The output path is derived with `with_extension`, so `ManoharNalluri.tex`
/// produces `ManoharNalluri.docx` — the same single, overwritten file the PDF
/// uses, never a timestamped sibling.
pub fn convert(tex_path: &Path) -> RvmResult<DocxResult> {
    convert_with(tex_path, &resolve_binary())
}

/// Resolve the converter: `RVM_PANDOC` if set and non-empty, else `pandoc`.
pub fn resolve_binary() -> String {
    match std::env::var(BINARY_ENV) {
        Ok(value) if !value.trim().is_empty() => value.trim().to_string(),
        _ => DEFAULT_BINARY.to_string(),
    }
}

/// Convert `tex_path` with an explicit converter binary.
///
/// Split from [`convert`] so tests can hand it a stub instead of mutating the
/// process environment, which is shared with every other test in the binary.
pub fn convert_with(tex_path: &Path, binary: &str) -> RvmResult<DocxResult> {
    let docx_path = tex_path.with_extension("docx");

    let mut command = Command::new(binary);
    command
        .arg(tex_path)
        .arg("--from")
        .arg("latex")
        .arg("--to")
        .arg("docx")
        .arg("--output")
        .arg(&docx_path);

    // Resolve `\includegraphics{photo.png}` against the resume's own directory
    // rather than RVM's working directory: the file being compiled may belong
    // to a branch the operator did not start in.
    if let Some(dir) = tex_path.parent().filter(|dir| !dir.as_os_str().is_empty()) {
        command.arg("--resource-path").arg(dir);
    }

    let output = command.output().map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => RvmError::Other(format!(
            "'{binary}' not found. Install it with `brew install pandoc`, or point \
             {BINARY_ENV} at a pandoc binary."
        )),
        _ => RvmError::Other(format!("Failed to run '{binary}': {e}")),
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(RvmError::Other(format!(
            "'{binary}' could not convert '{}': {}",
            tex_path.display(),
            tail(stderr.trim(), 300)
        )));
    }

    // pandoc exits 0 with warnings, and they are the only sign that a macro was
    // silently dropped, so they are surfaced rather than discarded.
    let warnings = String::from_utf8_lossy(&output.stderr)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect();

    Ok(DocxResult {
        docx_path,
        warnings,
    })
}

/// Keep the last `limit` characters, which is where pandoc puts the real error.
fn tail(text: &str, limit: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= limit {
        return text.to_string();
    }
    chars[chars.len() - limit..].iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// A stand-in for pandoc.
    ///
    /// It lifts `--output` and the input path by flag name rather than by
    /// position, so adding an argument to the real invocation cannot silently
    /// turn these tests into assertions about argument order. The body sees
    /// them as `$in` and `$out`.
    fn fake_pandoc(body: &str) -> (TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fake-pandoc.sh");
        let script = format!(
            "#!/bin/sh\nin=\"$1\"\nout=\"\"\nwhile [ $# -gt 0 ]; do\n  \
             if [ \"$1\" = \"--output\" ]; then shift; out=\"$1\"; fi\n  shift\ndone\n{body}\n"
        );
        std::fs::write(&path, script).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        (dir, path)
    }

    fn tex_fixture(dir: &TempDir) -> PathBuf {
        let tex = dir.path().join("resume.tex");
        std::fs::write(
            &tex,
            "\\documentclass{article}\n\\begin{document}hi\\end{document}\n",
        )
        .unwrap();
        tex
    }

    fn count_docx(dir: &TempDir) -> usize {
        std::fs::read_dir(dir.path())
            .unwrap()
            .filter(|entry| {
                entry
                    .as_ref()
                    .unwrap()
                    .path()
                    .extension()
                    .map(|ext| ext == "docx")
                    .unwrap_or(false)
            })
            .count()
    }

    #[test]
    fn test_output_path_mirrors_the_tex_stem() {
        let (dir, bin) = fake_pandoc("printf 'PK' > \"$out\"");
        let tex = tex_fixture(&dir);
        let result = convert_with(&tex, bin.to_str().unwrap()).unwrap();
        assert_eq!(result.docx_path.file_name().unwrap(), "resume.docx");
        assert_eq!(result.docx_path.parent().unwrap(), tex.parent().unwrap());
        assert!(result.docx_path.exists());
    }

    #[test]
    fn test_conversion_replaces_an_existing_docx() {
        // One file, overwritten in place — never a second copy beside it.
        let (dir, bin) = fake_pandoc("printf 'new' > \"$out\"");
        let tex = tex_fixture(&dir);
        let stale = dir.path().join("resume.docx");
        std::fs::write(&stale, "old").unwrap();

        convert_with(&tex, bin.to_str().unwrap()).unwrap();

        assert_eq!(std::fs::read_to_string(&stale).unwrap(), "new");
        assert_eq!(count_docx(&dir), 1, "a second .docx was left behind");
    }

    #[test]
    fn test_missing_binary_names_the_install_command() {
        let dir = tempfile::tempdir().unwrap();
        let tex = tex_fixture(&dir);
        let err = convert_with(&tex, "/nonexistent/pandoc-binary").unwrap_err();
        let text = format!("{err}");
        assert!(text.contains("not found"), "{text}");
        assert!(text.contains("brew install pandoc"), "{text}");
        assert!(text.contains(BINARY_ENV), "{text}");
    }

    #[test]
    fn test_failure_reports_pandoc_stderr() {
        let (dir, bin) = fake_pandoc("echo 'unexpected \\\\end{document}' >&2; exit 3");
        let tex = tex_fixture(&dir);
        let err = convert_with(&tex, bin.to_str().unwrap()).unwrap_err();
        let text = format!("{err}");
        assert!(text.contains("could not convert"), "{text}");
        assert!(text.contains("unexpected"), "{text}");
    }

    #[test]
    fn test_success_exposes_warnings() {
        let (dir, bin) =
            fake_pandoc("echo '[WARNING] Could not convert token' >&2; printf 'PK' > \"$out\"");
        let tex = tex_fixture(&dir);
        let result = convert_with(&tex, bin.to_str().unwrap()).unwrap();
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("Could not convert token"));
    }

    #[test]
    fn test_tex_path_is_passed_to_the_converter() {
        let (dir, bin) = fake_pandoc("printf '%s' \"$in\" > \"$out\"");
        let tex = tex_fixture(&dir);
        convert_with(&tex, bin.to_str().unwrap()).unwrap();
        let recorded = std::fs::read_to_string(dir.path().join("resume.docx")).unwrap();
        assert_eq!(recorded, tex.display().to_string());
    }

    #[test]
    fn test_tail_keeps_the_end() {
        assert_eq!(tail("abcdef", 3), "def");
        assert_eq!(tail("ab", 10), "ab");
    }
}
