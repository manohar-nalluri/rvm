use std::path::Path;

use rvm_types::{Diagnostic, RvmResult};

/// Check if the PDF exceeds the page limit.
pub fn check_page_count(pdf_path: &Path, limit: usize) -> RvmResult<Option<Diagnostic>> {
    let doc = lopdf::Document::load(pdf_path)
        .map_err(|e| rvm_types::RvmError::Other(format!("Failed to read PDF: {}", e)))?;

    let page_count = doc.get_pages().len();

    if page_count > limit {
        Ok(Some(
            Diagnostic::error(
                "E001",
                format!(
                    "Output is {} page(s); limit is {} page(s). Reduce content by approximately {}%.",
                    page_count,
                    limit,
                    ((page_count - limit) as f64 / page_count as f64 * 100.0) as u32
                ),
            )
            .with_suggestion("Remove less relevant experience entries or reduce bullet points."),
        ))
    } else {
        Ok(None)
    }
}
