use rvm_types::{Diagnostic, diagnostic::ValidationReport};

/// Validate that required sections are present in the .tex source.
pub fn validate_sections(
    tex_content: &str,
    required: &[String],
    report: &mut ValidationReport,
) {
    let lower = tex_content.to_lowercase();
    for section in required {
        if !lower.contains(&section.to_lowercase()) {
            report.push(
                Diagnostic::error(
                    "E010",
                    format!("Required section '{}' not found in resume.", section),
                )
                .with_suggestion(format!(
                    "Add a '{}' section to your resume.",
                    section
                )),
            );
        }
    }
}

/// Validate bullet point quality.
pub fn validate_bullets(tex_content: &str, report: &mut ValidationReport) {
    for line in tex_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("\\item") {
            let content = trimmed.trim_start_matches("\\item").trim();
            let word_count = content.split_whitespace().count();

            if word_count < 5 && !content.is_empty() {
                report.push(
                    Diagnostic::info(
                        "I020",
                        format!("Short bullet point ({} words): '{}'", word_count, content),
                    )
                    .with_suggestion("Expand with quantified impact or specific details."),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_sections_present() {
        let tex = "\\section{Experience}\n\\section{Education}\n\\section{Skills}";
        let required = vec![
            "experience".to_string(),
            "education".to_string(),
            "skills".to_string(),
        ];
        let mut report = ValidationReport::default();
        validate_sections(tex, &required, &mut report);
        assert!(!report.has_errors());
    }

    #[test]
    fn test_missing_section() {
        let tex = "\\section{Experience}";
        let required = vec!["experience".to_string(), "education".to_string()];
        let mut report = ValidationReport::default();
        validate_sections(tex, &required, &mut report);
        assert!(report.has_errors());
    }

    #[test]
    fn test_short_bullet_flagged() {
        let tex = "\\item Short note";
        let mut report = ValidationReport::default();
        validate_bullets(tex, &mut report);
        assert!(report.diagnostics.iter().any(|d| d.code == "I020"));
    }

    #[test]
    fn test_long_bullet_not_flagged() {
        let tex = "\\item Designed and implemented a scalable microservices architecture serving millions of requests";
        let mut report = ValidationReport::default();
        validate_bullets(tex, &mut report);
        assert!(report.diagnostics.is_empty());
    }
}
