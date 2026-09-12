use std::path::{Path, PathBuf};
use std::process::Command;

use rvm_types::{RvmError, RvmResult};

use crate::error::CompileError;

pub struct CompileResult {
    pub pdf_path: PathBuf,
    pub warnings: Vec<String>,
}

pub struct Compiler {
    engine: EngineKind,
}

#[derive(Debug, Clone)]
enum EngineKind {
    Tectonic,
    Latexmk,
}

impl Compiler {
    pub fn new(engine_name: &str) -> RvmResult<Self> {
        let engine = match engine_name {
            "tectonic" => EngineKind::Tectonic,
            "latexmk" => EngineKind::Latexmk,
            other => {
                return Err(RvmError::ConfigError(format!(
                    "Unknown compiler engine: '{}'. Use 'tectonic' or 'latexmk'.",
                    other
                )))
            }
        };
        Ok(Self { engine })
    }

    /// Compile a .tex file to PDF.
    pub fn compile(&self, tex_path: &Path) -> RvmResult<CompileResult> {
        let output_dir = tex_path.parent().unwrap_or(Path::new("."));

        let result = match &self.engine {
            EngineKind::Tectonic => self.compile_tectonic(tex_path, output_dir),
            EngineKind::Latexmk => self.compile_latexmk(tex_path, output_dir),
        }?;

        Ok(result)
    }

    fn compile_tectonic(&self, tex_path: &Path, output_dir: &Path) -> RvmResult<CompileResult> {
        let output = Command::new("tectonic")
            .arg("-X")
            .arg("compile")
            .arg(tex_path)
            .arg("--outdir")
            .arg(output_dir)
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    RvmError::CompilationFailed(
                        "tectonic not found. Install it with: brew install tectonic".to_string(),
                    )
                } else {
                    RvmError::CompilationFailed(format!("Failed to run tectonic: {}", e))
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RvmError::CompilationFailed(
                CompileError::parse_latex_errors(&stderr),
            ));
        }

        let pdf_name = tex_path.with_extension("pdf");
        let pdf_path = output_dir.join(pdf_name.file_name().unwrap());

        let warnings = CompileError::parse_latex_warnings(
            &String::from_utf8_lossy(&output.stderr),
        );

        Ok(CompileResult {
            pdf_path,
            warnings,
        })
    }

    fn compile_latexmk(&self, tex_path: &Path, output_dir: &Path) -> RvmResult<CompileResult> {
        let output = Command::new("latexmk")
            .arg("-pdf")
            .arg("-interaction=nonstopmode")
            .arg(format!("-outdir={}", output_dir.display()))
            .arg(tex_path)
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    RvmError::CompilationFailed(
                        "latexmk not found. Install it with: brew install --cask mactex-no-gui (or brew install tectonic for a lighter alternative)".to_string(),
                    )
                } else {
                    RvmError::CompilationFailed(format!("Failed to run latexmk: {}", e))
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(RvmError::CompilationFailed(
                CompileError::parse_latex_errors(&format!("{}\n{}", stdout, stderr)),
            ));
        }

        let pdf_name = tex_path.with_extension("pdf");
        let pdf_path = output_dir.join(pdf_name.file_name().unwrap());

        Ok(CompileResult {
            pdf_path,
            warnings: Vec::new(),
        })
    }
}
