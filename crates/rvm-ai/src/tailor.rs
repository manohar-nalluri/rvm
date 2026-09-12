/// Input context for AI resume tailoring.
#[derive(Debug, Clone)]
pub struct TailorContext {
    pub current_tex: String,
    pub job_description: String,
    pub company_name: Option<String>,
    pub role: Option<String>,
}

/// Output from AI tailoring.
#[derive(Debug, Clone)]
pub struct TailorResult {
    pub tailored_tex: String,
    pub changes_summary: String,
    pub keywords_added: Vec<String>,
}

/// Build the prompt for AI resume tailoring.
pub fn build_tailor_prompt(ctx: &TailorContext) -> String {
    let mut prompt = String::new();
    prompt.push_str("You are an expert resume writer. Tailor the following LaTeX resume ");
    prompt.push_str("to maximize relevance for the given job description.\n\n");

    if let Some(company) = &ctx.company_name {
        prompt.push_str(&format!("Company: {}\n", company));
    }
    if let Some(role) = &ctx.role {
        prompt.push_str(&format!("Role: {}\n", role));
    }

    prompt.push_str("\n--- JOB DESCRIPTION ---\n");
    prompt.push_str(&ctx.job_description);
    prompt.push_str("\n\n--- CURRENT RESUME (.tex) ---\n");
    prompt.push_str(&ctx.current_tex);
    prompt.push_str("\n\n--- INSTRUCTIONS ---\n");
    prompt.push_str("1. Rewrite bullet points to include keywords from the JD\n");
    prompt.push_str("2. Reorder sections to highlight the most relevant experience\n");
    prompt.push_str("3. Preserve all LaTeX formatting and structure\n");
    prompt.push_str("4. Keep the resume to 1 page\n");
    prompt.push_str("5. Output ONLY the complete .tex file content\n");

    prompt
}

/// Extract keywords from a job description (deduplicated, case-insensitive).
pub fn extract_keywords(jd_text: &str) -> Vec<String> {
    use std::collections::HashSet;

    let stop_words: &[&str] = &[
        "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
        "of", "with", "by", "is", "are", "was", "were", "be", "been", "being",
        "have", "has", "had", "do", "does", "did", "will", "would", "could",
        "should", "may", "might", "shall", "can", "need", "must", "we", "you",
        "they", "our", "your", "their", "this", "that", "these", "those",
        "it", "its", "from", "as", "not", "also", "about", "into", "through",
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
        let python_count = keywords.iter().filter(|k| k.to_lowercase() == "python").count();
        assert_eq!(python_count, 1);
    }

    #[test]
    fn test_extract_keywords_preserves_special_chars() {
        let keywords = extract_keywords("Node.js and C++ developer");
        assert!(keywords.iter().any(|k| k == "C++"));
        assert!(keywords.iter().any(|k| k == "Node.js"));
    }

    #[test]
    fn test_build_tailor_prompt_includes_context() {
        let ctx = TailorContext {
            current_tex: "\\documentclass{article}".to_string(),
            job_description: "Looking for a Rust developer".to_string(),
            company_name: Some("Acme Corp".to_string()),
            role: Some("Software Engineer".to_string()),
        };
        let prompt = build_tailor_prompt(&ctx);
        assert!(prompt.contains("Acme Corp"));
        assert!(prompt.contains("Software Engineer"));
        assert!(prompt.contains("Rust developer"));
        assert!(prompt.contains("\\documentclass"));
    }
}
