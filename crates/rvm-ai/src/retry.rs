use rvm_types::diagnostic::ValidationReport;

/// Context passed to the AI for a retry attempt after validation failure.
#[derive(Debug)]
pub struct RetryContext {
    pub attempt: u8,
    pub max_retries: u8,
    pub current_tex: String,
    pub diagnostics: Vec<String>,
}

impl RetryContext {
    pub fn from_report(
        attempt: u8,
        max_retries: u8,
        current_tex: String,
        report: &ValidationReport,
    ) -> Self {
        let diagnostics = report
            .diagnostics
            .iter()
            .map(|d| d.to_string())
            .collect();

        Self {
            attempt,
            max_retries,
            current_tex,
            diagnostics,
        }
    }

    /// Build a retry prompt including the validation diagnostics.
    pub fn build_retry_prompt(&self) -> String {
        let mut prompt = String::new();
        prompt.push_str(&format!(
            "Retry attempt {}/{} - Fix the following validation issues:\n\n",
            self.attempt, self.max_retries
        ));

        for (i, diag) in self.diagnostics.iter().enumerate() {
            prompt.push_str(&format!("{}. {}\n", i + 1, diag));
        }

        prompt.push_str("\n--- CURRENT RESUME (.tex) ---\n");
        prompt.push_str(&self.current_tex);
        prompt.push_str("\n\n--- INSTRUCTIONS ---\n");
        prompt.push_str("Fix ALL the above issues and output the corrected .tex file.\n");
        prompt.push_str("Preserve all LaTeX structure and formatting.\n");
        prompt
    }

    /// Check if we've exhausted all retries.
    pub fn exhausted(&self) -> bool {
        self.attempt >= self.max_retries
    }
}
