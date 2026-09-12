pub mod ats;
pub mod content;
pub mod page;
pub mod plugin;
pub mod whitespace;

use std::path::Path;

use rvm_types::{DiagnosticTier, RvmConfig, RvmResult, diagnostic::ValidationReport};

/// Run the full validation suite against a compiled PDF and its source .tex.
pub fn validate(
    pdf_path: &Path,
    tex_content: &str,
    config: &RvmConfig,
) -> RvmResult<ValidationReport> {
    let mut report = ValidationReport::default();

    // Page count validation
    if let Some(diag) = page::check_page_count(pdf_path, config.document.page_limit)? {
        report.push(diag);
    }

    // Content validation
    content::validate_sections(tex_content, &config.document.required_sections, &mut report);
    content::validate_bullets(tex_content, &mut report);

    // ATS compatibility checks
    if config.validation.ats_check {
        ats::check_compatibility(tex_content, &mut report);
    }

    // In strict mode, promote warnings to errors
    if config.validation.strict_mode {
        for diag in &mut report.diagnostics {
            if diag.tier == DiagnosticTier::Warning {
                diag.tier = DiagnosticTier::Error;
            }
        }
    }

    Ok(report)
}
