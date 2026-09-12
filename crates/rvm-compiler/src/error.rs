/// Utilities for parsing LaTeX compiler output into human-readable diagnostics.
pub struct CompileError;

impl CompileError {
    /// Parse LaTeX error output into a user-friendly message.
    pub fn parse_latex_errors(raw: &str) -> String {
        let mut errors = Vec::new();

        for line in raw.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('!') {
                errors.push(trimmed.trim_start_matches('!').trim().to_string());
            } else if trimmed.contains("Fatal error") {
                errors.push(trimmed.to_string());
            }
        }

        if errors.is_empty() {
            raw.to_string()
        } else {
            errors.join("\n")
        }
    }

    /// Extract warnings from LaTeX output.
    pub fn parse_latex_warnings(raw: &str) -> Vec<String> {
        raw.lines()
            .filter(|line| {
                let lower = line.to_lowercase();
                lower.contains("warning") || lower.contains("overfull") || lower.contains("underfull")
            })
            .map(|s| s.trim().to_string())
            .collect()
    }
}
