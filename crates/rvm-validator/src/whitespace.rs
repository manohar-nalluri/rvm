use rvm_types::Diagnostic;

/// Analyze the trailing whitespace on the last page.
/// This is a placeholder - full implementation requires PDF rendering.
pub fn check_whitespace(whitespace_percent: f64, max_percent: u8) -> Option<Diagnostic> {
    if whitespace_percent > max_percent as f64 {
        Some(
            Diagnostic::warning(
                "W001",
                format!(
                    "Last page has {:.0}% empty space (threshold: {}%). Consider adding content.",
                    whitespace_percent, max_percent
                ),
            )
            .with_suggestion(
                "Add more bullet points, a projects section, or adjust spacing to fill the page.",
            ),
        )
    } else {
        None
    }
}
