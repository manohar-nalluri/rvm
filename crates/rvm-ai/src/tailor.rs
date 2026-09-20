//! Resume tailoring: build the prompt, splice the model's answer back into the
//! operator's LaTeX, and refuse anything that is not a usable resume.
//!
//! The model is asked for the **body only** — the text between
//! `\begin{document}` and `\end{document}` — and RVM splices it into a
//! byte-exact copy of the operator's own preamble. Two things fall out of that:
//! the ATS notes and package options that live in the preamble cannot be
//! silently dropped or "improved" by the model, and a model that returns the
//! whole document anyway is still handled by unwrapping it in
//! [`extract_body`].

use rvm_types::{RvmError, RvmResult};

pub const BEGIN_DOC: &str = "\\begin{document}";
pub const END_DOC: &str = "\\end{document}";

/// Everything a tailoring call needs to know.
#[derive(Debug, Clone)]
pub struct TailorRequest {
    pub source_tex: String,
    pub job_description: String,
    pub company: Option<String>,
    pub role: Option<String>,
    /// Terms that must survive tailoring even though the tailorer is otherwise
    /// forbidden from introducing technology not already in the resume.
    pub required_terms: Vec<String>,
    /// Workspace-specific house rules appended to the prompt.
    pub extra_instructions: String,
}

/// A source `.tex` split around its document body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    /// Preamble, up to and including `\begin{document}`.
    pub head: String,
    /// The body the model is allowed to rewrite.
    pub body: String,
    /// `\end{document}` and anything after it.
    pub tail: String,
}

impl Document {
    pub fn parse(tex: &str) -> RvmResult<Self> {
        let start = tex.find(BEGIN_DOC).ok_or_else(|| {
            RvmError::AiError(format!("the source resume has no {BEGIN_DOC} marker"))
        })?;
        let head_end = start + BEGIN_DOC.len();

        let end = tex.rfind(END_DOC).ok_or_else(|| {
            RvmError::AiError(format!("the source resume has no {END_DOC} marker"))
        })?;
        if end < head_end {
            return Err(RvmError::AiError(format!(
                "{END_DOC} appears before {BEGIN_DOC} in the source resume"
            )));
        }

        Ok(Self {
            head: tex[..head_end].to_string(),
            body: tex[head_end..end].to_string(),
            tail: tex[end..].to_string(),
        })
    }

    /// Rebuild a full document around a replacement body.
    pub fn render(&self, body: &str) -> String {
        let mut out = String::with_capacity(self.head.len() + body.len() + self.tail.len() + 4);
        out.push_str(&self.head);
        if !self.head.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(body.trim_matches('\n'));
        out.push('\n');
        out.push_str(&self.tail);
        out
    }
}

/// Build the first-attempt prompt.
pub fn build_prompt(req: &TailorRequest) -> RvmResult<String> {
    let doc = Document::parse(&req.source_tex)?;
    let mut prompt = String::new();

    prompt.push_str(
        "You are a text transformation function, not a coding agent. You have NO tools \
         available and you must not attempt to use any: do not read files, do not write files, \
         do not run commands, do not search. Everything you need is in this message. Reply with \
         the transformed text directly in your response.\n\n",
    );
    prompt.push_str(
        "TASK: Rewrite the BODY of a LaTeX resume so it targets the job description below.\n\n",
    );
    prompt.push_str("OUTPUT FORMAT - follow exactly:\n");
    prompt.push_str(
        "- Reply with ONLY the rewritten body: raw LaTeX, no markdown fences, no commentary, \
         no explanation, no tool calls.\n",
    );
    prompt.push_str(&format!(
        "- Do NOT output \\documentclass, the preamble, {BEGIN_DOC} or {END_DOC}.\n"
    ));
    prompt.push_str(
        "- Start directly with the first content line of the body and end with the last line of \
         the final section. Never stop mid-section.\n\n",
    );

    prompt.push_str("ABSOLUTE INTEGRITY RULES - violating any of these makes the output useless:\n");
    prompt.push_str(
        "1. The skills section may only REORDER and REWORD technologies already present in the \
         source body. Do not add a single language, framework, database, cloud service, platform \
         or tool that is not already in the source body.\n",
    );
    prompt.push_str(
        "2. Do not invent metrics, percentages, scale figures or numbers that are not already in \
         the source body.\n",
    );
    prompt.push_str(
        "3. Do not invent employers, job titles, dates, degrees or certifications.\n",
    );
    prompt.push_str(
        "4. Keep roughly the same number of bullets. Rewrite existing bullets; do not pad the \
         list with new ones.\n",
    );
    prompt.push_str("5. Never mention the hiring company's name anywhere in the resume.\n");
    prompt.push_str(
        "6. Do not change fonts, margins, spacing, package options, section names or command \
         names.\n",
    );
    prompt.push_str("7. Only use LaTeX commands that already appear in the source body.\n");
    if !req.required_terms.is_empty() {
        prompt.push_str(&format!(
            "8. The skills section MUST list all of these terms: {}.\n",
            req.required_terms.join(", ")
        ));
    }

    prompt.push_str("\nSTYLE RULES:\n");
    prompt.push_str("- Start each bullet with a strong past-tense action verb.\n");
    prompt.push_str("- Bold technology names and key concepts with \\textbf{}.\n");
    prompt.push_str(
        "- Mirror the job description's exact keyword phrases naturally inside existing bullets.\n",
    );
    prompt.push_str(
        "- Reorder bullets and skill lines so the most job-relevant content comes first.\n",
    );
    prompt.push_str("- The compiled document must still fit on exactly ONE page.\n");

    if !req.extra_instructions.trim().is_empty() {
        prompt.push_str("\nWORKSPACE HOUSE RULES (also mandatory):\n");
        prompt.push_str(req.extra_instructions.trim());
        prompt.push('\n');
    }

    if req.company.is_some() || req.role.is_some() {
        prompt.push_str("\n--- TARGET ROLE ---\n");
        if let Some(company) = &req.company {
            prompt.push_str(&format!("Company (must NOT appear in the resume): {company}\n"));
        }
        if let Some(role) = &req.role {
            prompt.push_str(&format!("Role title: {role}\n"));
        }
    }

    prompt.push_str("\n--- JOB DESCRIPTION ---\n");
    prompt.push_str(req.job_description.trim());
    prompt.push_str("\n\n--- SOURCE BODY (.tex body only) ---\n");
    prompt.push_str(doc.body.trim());
    prompt.push_str("\n\n--- REPLY WITH THE REWRITTEN BODY ONLY ---\n");

    Ok(prompt)
}

/// Build a repair prompt that shows the model what was wrong last time.
pub fn build_repair_prompt(req: &TailorRequest, reason: &str) -> RvmResult<String> {
    let doc = Document::parse(&req.source_tex)?;
    let mut prompt = String::new();

    prompt.push_str(
        "You are a text transformation function, not a coding agent. You have NO tools and must \
         not attempt to use any. Reply with the transformed text directly.\n\n",
    );
    prompt.push_str(&format!(
        "Your previous answer could not be used. Reason: {}\n\n",
        reason.trim()
    ));
    prompt.push_str(
        "Reply again with ONLY the rewritten LaTeX body (raw LaTeX, no fences, no commentary, \
         no tool calls). Do not output \\documentclass, the preamble, ",
    );
    prompt.push_str(BEGIN_DOC);
    prompt.push_str(" or ");
    prompt.push_str(END_DOC);
    prompt.push_str(". Reply with the complete body from first line to last - do not truncate.\n");
    prompt.push_str(
        "\nRepeat of the hard rules: do not add technologies, metrics, employers, titles or \
         dates that are not in the source; never name the hiring company; keep every section; \
         output nothing but the body.\n",
    );

    prompt.push_str("\n--- JOB DESCRIPTION ---\n");
    prompt.push_str(req.job_description.trim());
    prompt.push_str("\n\n--- SOURCE BODY (.tex body only) ---\n");
    prompt.push_str(doc.body.trim());
    prompt.push_str("\n\n--- REPLY WITH THE REWRITTEN BODY ONLY ---\n");

    Ok(prompt)
}

/// Unwrap the model's answer into a bare body.
pub fn extract_body(raw: &str) -> RvmResult<String> {
    let cleaned = strip_fences(raw);
    let text = cleaned.trim();
    if text.is_empty() {
        return Err(RvmError::AiError(
            "the model returned an empty response".to_string(),
        ));
    }

    // Take the last document, not the first: a model that reasons out loud may
    // echo the source document before producing its own.
    let after_begin = match text.rfind(BEGIN_DOC) {
        Some(idx) => &text[idx + BEGIN_DOC.len()..],
        None => text,
    };
    let body = match after_begin.rfind(END_DOC) {
        Some(idx) => &after_begin[..idx],
        None => after_begin,
    };

    let body = body.trim_matches('\n').trim_end();
    if body.is_empty() {
        return Err(RvmError::AiError(
            "the model returned a document with an empty body".to_string(),
        ));
    }
    Ok(body.to_string())
}

/// Drop markdown fence lines so a fenced answer is still usable.
fn strip_fences(raw: &str) -> String {
    raw.lines()
        .filter(|line| !line.trim_start().starts_with("```"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Reject an answer that is structurally unusable *before* spending a compile
/// on it, and feed the reason into the repair prompt.
///
/// Everything checked here is a hard fail: a resume that is truncated, that has
/// lost a section, or that has quietly reintroduced a preamble is worse than no
/// resume at all, because it would be committed and later applied with.
pub fn lint(
    tex: &str,
    source_body: &str,
    required_terms: &[String],
    required_sections: &[String],
) -> RvmResult<()> {
    let doc = Document::parse(tex)?;
    let body = doc.body.trim();

    if body.contains("\\documentclass") {
        return Err(RvmError::AiError(
            "the answer included a preamble; only the document body is allowed".to_string(),
        ));
    }
    if body.contains(BEGIN_DOC) {
        return Err(RvmError::AiError(
            format!("the answer included a nested {BEGIN_DOC}"),
        ));
    }
    if body.len() < 200 {
        return Err(RvmError::AiError(format!(
            "the rewritten body is only {} bytes, which cannot be a whole resume",
            body.len()
        )));
    }

    let source_len = source_body.trim().len().max(1);
    let ratio = body.len() as f64 / source_len as f64;
    if ratio < 0.4 {
        return Err(RvmError::AiError(format!(
            "the rewritten body is {:.0}% of the source length, so it was truncated",
            ratio * 100.0
        )));
    }

    if !braces_balanced(body) {
        return Err(RvmError::AiError(
            "the rewritten body has unbalanced braces, so it was truncated or corrupted"
                .to_string(),
        ));
    }

    let missing: Vec<&String> = required_terms
        .iter()
        .filter(|term| {
            !term.trim().is_empty() && !body.to_lowercase().contains(&term.trim().to_lowercase())
        })
        .collect();
    if !missing.is_empty() {
        let names: Vec<&str> = missing.iter().map(|s| s.as_str()).collect();
        return Err(RvmError::AiError(format!(
            "the rewritten body is missing required terms: {}",
            names.join(", ")
        )));
    }

    let lower = body.to_lowercase();
    let lost: Vec<&String> = required_sections
        .iter()
        .filter(|section| {
            !section.trim().is_empty() && !lower.contains(&section.trim().to_lowercase())
        })
        .collect();
    if !lost.is_empty() {
        let names: Vec<&str> = lost.iter().map(|s| s.as_str()).collect();
        return Err(RvmError::AiError(format!(
            "the rewritten body dropped required section(s): {}",
            names.join(", ")
        )));
    }

    Ok(())
}

/// Whether `{` and `}` balance, ignoring escaped braces.
fn braces_balanced(body: &str) -> bool {
    let mut depth: i64 = 0;
    let chars: Vec<char> = body.chars().collect();
    for (i, ch) in chars.iter().enumerate() {
        match ch {
            '{' => {
                if !escaped(&chars, i) {
                    depth += 1;
                }
            }
            '}' => {
                if !escaped(&chars, i) {
                    depth -= 1;
                    if depth < 0 {
                        return false;
                    }
                }
            }
            _ => {}
        }
    }
    depth == 0
}

fn escaped(chars: &[char], index: usize) -> bool {
    let mut backslashes = 0;
    let mut i = index;
    while i > 0 {
        i -= 1;
        if chars[i] == '\\' {
            backslashes += 1;
        } else {
            break;
        }
    }
    backslashes % 2 == 1
}

/// Deterministic, filesystem- and git-safe branch name for a job.
///
/// Stable across runs on purpose: re-tailoring the same job must update the
/// branch the database already points at, not create a second one.
pub fn slugify(company: Option<&str>, role: Option<&str>, fallback_seed: Option<&str>) -> String {
    let mut parts: Vec<String> = Vec::new();
    for value in [company, role].into_iter().flatten() {
        let slug = slug(value);
        if !slug.is_empty() {
            parts.push(slug);
        }
    }
    if parts.is_empty() {
        if let Some(seed) = fallback_seed {
            let slug = slug(seed);
            if !slug.is_empty() {
                parts.push(slug);
            }
        }
    }
    let mut name = parts.join("-");
    name.truncate(48);
    while name.ends_with('-') {
        name.pop();
    }
    if name.is_empty() {
        name = "tailored".to_string();
    }
    name
}

fn slug(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut pending_dash = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            pending_dash = false;
            out.push(ch.to_ascii_lowercase());
        } else {
            pending_dash = true;
        }
    }
    out
}

/// Extract keywords from a job description (deduplicated, case-insensitive).
pub fn extract_keywords(jd_text: &str) -> Vec<String> {
    use std::collections::HashSet;

    let stop_words: &[&str] = &[
        "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
        "is", "are", "was", "were", "be", "been", "being", "have", "has", "had", "do", "does",
        "did", "will", "would", "could", "should", "may", "might", "shall", "can", "need", "must",
        "we", "you", "they", "our", "your", "their", "this", "that", "these", "those", "it", "its",
        "from", "as", "not", "also", "about", "into", "through",
    ];

    let mut seen = HashSet::new();
    jd_text
        .split(|c: char| !c.is_alphanumeric() && c != '+' && c != '#' && c != '.')
        .filter(|word| {
            let lower = word.to_lowercase();
            word.len() > 2 && !stop_words.contains(&lower.as_str()) && seen.insert(lower)
        })
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "\\documentclass{article}\n\\begin{document}\n\\section{Skills}\nBody line one here.\n\\end{document}\n";

    fn request() -> TailorRequest {
        TailorRequest {
            source_tex: SOURCE.to_string(),
            job_description: "We need a Rust developer".to_string(),
            company: Some("Acme Corp".to_string()),
            role: Some("Software Engineer".to_string()),
            required_terms: Vec::new(),
            extra_instructions: String::new(),
        }
    }

    #[test]
    fn test_document_parse_splits_around_body() {
        let doc = Document::parse(SOURCE).unwrap();
        assert_eq!(doc.head, "\\documentclass{article}\n\\begin{document}");
        assert_eq!(doc.body, "\n\\section{Skills}\nBody line one here.\n");
        assert_eq!(doc.tail, "\\end{document}\n");
    }

    #[test]
    fn test_document_render_round_trips_the_source() {
        let doc = Document::parse(SOURCE).unwrap();
        let rendered = doc.render(&doc.body);
        // Whitespace at the seam is normalised, nothing else moves.
        assert!(rendered.starts_with("\\documentclass{article}\n\\begin{document}\n"));
        assert!(rendered.ends_with("\\end{document}\n"));
        assert!(rendered.contains("Body line one here."));
    }

    #[test]
    fn test_document_parse_requires_markers() {
        assert!(Document::parse("no markers here").is_err());
        assert!(Document::parse("\\begin{document}\n\\end{document}").is_ok());
    }

    #[test]
    fn test_render_preserves_operator_preamble_byte_for_byte() {
        let source = "% ATS NOTES (do not remove)\n\\documentclass[11pt]{article}\n\\begin{document}\nold\n\\end{document}\n";
        let doc = Document::parse(source).unwrap();
        let rendered = doc.render("new body");
        assert!(rendered.starts_with("% ATS NOTES (do not remove)\n\\documentclass[11pt]{article}\n\\begin{document}\n"));
        assert!(rendered.contains("new body"));
        assert!(!rendered.contains("old"));
    }

    #[test]
    fn test_extract_body_accepts_bare_body() {
        assert_eq!(extract_body("\\section{Skills}\nhi").unwrap(), "\\section{Skills}\nhi");
    }

    #[test]
    fn test_extract_body_unwraps_a_full_document() {
        let answer = "Sure!\n```latex\n\\documentclass{article}\n\\begin{document}\nBODY HERE\n\\end{document}\n```\n";
        assert_eq!(extract_body(answer).unwrap(), "BODY HERE");
    }

    #[test]
    fn test_extract_body_rejects_empty() {
        assert!(extract_body("   ").is_err());
        assert!(extract_body("\\begin{document}\n\\end{document}").is_err());
    }

    #[test]
    fn test_lint_rejects_preamble_in_body() {
        let tex = "\\documentclass{article}\n\\begin{document}\n\\documentclass{article}\n\\end{document}";
        assert!(lint(tex, "x", &[], &[]).is_err());
    }

    #[test]
    fn test_lint_rejects_truncation() {
        let doc = Document::parse(SOURCE).unwrap();
        let short = doc.render("too short");
        assert!(lint(&short, &doc.body, &[], &[]).is_err());
    }

    #[test]
    fn test_lint_rejects_unbalanced_braces() {
        let doc = Document::parse(SOURCE).unwrap();
        let long = "\\textbf{oops ".to_string() + &"word ".repeat(200);
        let tex = doc.render(&long);
        assert!(lint(&tex, &doc.body, &[], &[]).is_err());
    }

    #[test]
    fn test_lint_accepts_a_plausible_rewrite() {
        let doc = Document::parse(SOURCE).unwrap();
        let good = doc.render(&format!("\\section{{Skills}}\n{}", "Built things with Python. ".repeat(20)));
        assert!(lint(&good, &doc.body, &[], &[]).is_ok());
    }

    #[test]
    fn test_lint_enforces_required_terms() {
        let doc = Document::parse(SOURCE).unwrap();
        let good = doc.render(&format!("\\section{{Skills}}\n{}", "Built things with Python. ".repeat(20)));
        assert!(lint(&good, &doc.body, &["C++".to_string()], &[]).is_err());
    }

    #[test]
    fn test_lint_enforces_required_sections() {
        let doc = Document::parse(SOURCE).unwrap();
        let good = doc.render(&format!("\\section{{Skills}}\n{}", "Built things with Python. ".repeat(20)));
        assert!(lint(&good, &doc.body, &[], &["skills".to_string()]).is_ok());
        assert!(lint(&good, &doc.body, &[], &["education".to_string()]).is_err());
    }

    #[test]
    fn test_braces_balanced_ignores_escaped_braces() {
        assert!(braces_balanced("\\{ \\}"));
        assert!(!braces_balanced("{"));
        assert!(!braces_balanced("}"));
    }

    #[test]
    fn test_prompt_asks_for_body_only_and_forbids_tools() {
        let prompt = build_prompt(&request()).unwrap();
        assert!(prompt.contains("NO tools"));
        assert!(prompt.contains(BEGIN_DOC));
        assert!(prompt.contains("Acme Corp"));
        assert!(prompt.contains("Rust developer"));
        assert!(prompt.contains("SOURCE BODY"));
        // The source *body* is inlined; the preamble must not be.
        assert!(!prompt.contains("\\documentclass{article}\n\\begin{document}"));
    }

    #[test]
    fn test_prompt_includes_required_terms_and_house_rules() {
        let mut req = request();
        req.required_terms = vec!["C".to_string(), "C++".to_string()];
        req.extra_instructions = "Always keep the phone number.".to_string();
        let prompt = build_prompt(&req).unwrap();
        assert!(prompt.contains("C, C++"));
        assert!(prompt.contains("Always keep the phone number."));
    }

    #[test]
    fn test_repair_prompt_carries_the_reason() {
        let prompt = build_repair_prompt(&request(), "output was 2 pages").unwrap();
        assert!(prompt.contains("output was 2 pages"));
    }

    #[test]
    fn test_slugify_is_deterministic_and_safe() {
        assert_eq!(
            slugify(Some("Acme Corp."), Some("Sr. Backend/SWE"), None),
            slugify(Some("Acme Corp."), Some("Sr. Backend/SWE"), None)
        );
        let slug = slugify(Some("Acme Corp."), Some("Sr. Backend/SWE"), None);
        assert!(slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'), "{slug}");
        assert_eq!(slug, "acme-corp-sr-backend-swe");
    }

    #[test]
    fn test_slugify_falls_back_when_names_are_missing() {
        assert_eq!(slugify(None, None, Some("4469578911")), "4469578911");
        assert_eq!(slugify(Some("   "), Some("!!!"), None), "tailored");
    }

    #[test]
    fn test_slugify_caps_length_without_a_trailing_dash() {
        let slug = slugify(Some(&"a".repeat(40)), Some(&"b".repeat(40)), None);
        assert!(slug.len() <= 48, "{slug}");
        assert!(!slug.ends_with('-'));
    }

    #[test]
    fn test_extract_keywords_filters_stop_words() {
        let keywords = extract_keywords("We are looking for a software engineer with experience");
        assert!(!keywords.iter().any(|k| k == "are" || k == "for"));
        assert!(keywords.iter().any(|k| k == "software"));
        assert!(keywords.iter().any(|k| k == "engineer"));
    }

    #[test]
    fn test_extract_keywords_deduplicates() {
        let keywords = extract_keywords("Python python PYTHON developer");
        let python_count = keywords
            .iter()
            .filter(|k| k.to_lowercase() == "python")
            .count();
        assert_eq!(python_count, 1);
    }

    #[test]
    fn test_extract_keywords_preserves_special_chars() {
        let keywords = extract_keywords("Node.js and C++ developer");
        assert!(keywords.iter().any(|k| k == "C++"));
        assert!(keywords.iter().any(|k| k == "Node.js"));
    }
}
