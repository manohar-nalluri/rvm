use rvm_types::{Diagnostic, diagnostic::ValidationReport};

/// Check for ATS compatibility issues in the LaTeX source.
pub fn check_compatibility(tex_content: &str, report: &mut ValidationReport) {
    // Check for multi-column layouts
    if tex_content.contains("\\begin{multicols}") || tex_content.contains("\\begin{paracol}") {
        report.push(
            Diagnostic::warning("W010", "Multi-column layout detected. Many ATS parsers cannot handle multi-column content.")
                .with_suggestion("Use a single-column layout for better ATS compatibility."),
        );
    }

    // Check for tables used as layout
    if tex_content.contains("\\begin{tabular") && !tex_content.contains("\\begin{table}") {
        report.push(
            Diagnostic::warning(
                "W011",
                "Tables used for layout detected. ATS parsers may misread table-based layouts.",
            )
            .with_suggestion("Use LaTeX spacing commands instead of tables for layout."),
        );
    }

    // Check for images
    if tex_content.contains("\\includegraphics") {
        report.push(
            Diagnostic::warning(
                "W012",
                "Embedded images detected. ATS systems cannot parse graphical content.",
            )
            .with_suggestion("Remove images or provide text alternatives."),
        );
    }

    // Check for standard section headings
    let standard_headings = [
        "experience",
        "education",
        "skills",
        "projects",
        "summary",
        "objective",
        "certifications",
    ];

    let lower = tex_content.to_lowercase();
    let has_any_heading = standard_headings.iter().any(|h| lower.contains(h));

    if !has_any_heading {
        report.push(
            Diagnostic::info(
                "I010",
                "No standard section headings found. ATS systems look for recognized headings.",
            )
            .with_suggestion("Use standard headings like Experience, Education, Skills."),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_resume_no_warnings() {
        let tex = r#"\section{Experience}
\section{Education}
\section{Skills}"#;
        let mut report = ValidationReport::default();
        check_compatibility(tex, &mut report);
        assert!(report.diagnostics.is_empty());
    }

    #[test]
    fn test_multicol_warning() {
        let tex = r#"\begin{multicols}{2}
\section{Experience}
\end{multicols}"#;
        let mut report = ValidationReport::default();
        check_compatibility(tex, &mut report);
        assert!(report.diagnostics.iter().any(|d| d.code == "W010"));
    }

    #[test]
    fn test_image_warning() {
        let tex = r#"\section{Experience}
\includegraphics{photo.png}"#;
        let mut report = ValidationReport::default();
        check_compatibility(tex, &mut report);
        assert!(report.diagnostics.iter().any(|d| d.code == "W012"));
    }

    #[test]
    fn test_no_standard_headings() {
        let tex = r#"\section{My Stuff}
Some content here"#;
        let mut report = ValidationReport::default();
        check_compatibility(tex, &mut report);
        assert!(report.diagnostics.iter().any(|d| d.code == "I010"));
    }
}
