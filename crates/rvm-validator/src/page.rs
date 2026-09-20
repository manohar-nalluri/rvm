use std::path::Path;

use rvm_types::{Diagnostic, RvmResult};

/// Number of pages in a compiled PDF.
///
/// Split out from [`check_page_count`] because callers that report a result
/// (rather than merely gate on it) need the count itself.
pub fn page_count(pdf_path: &Path) -> RvmResult<usize> {
    let doc = lopdf::Document::load(pdf_path)
        .map_err(|e| rvm_types::RvmError::Other(format!("Failed to read PDF: {}", e)))?;
    Ok(doc.get_pages().len())
}

/// Check if the PDF exceeds the page limit.
pub fn check_page_count(pdf_path: &Path, limit: usize) -> RvmResult<Option<Diagnostic>> {
    let page_count = page_count(pdf_path)?;

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
