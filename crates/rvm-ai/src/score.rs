/// Result of scoring a resume against a job description.
#[derive(Debug, Clone)]
pub struct ScoreResult {
    pub overall_score: f64,
    pub keyword_match_rate: f64,
    pub matched_keywords: Vec<String>,
    pub missing_keywords: Vec<String>,
    pub gap_analysis: Vec<String>,
}

/// Build the prompt for AI resume scoring.
pub fn build_score_prompt(tex_content: &str, jd_text: &str) -> String {
    let mut prompt = String::new();
    prompt.push_str("Analyze the following resume against the job description.\n");
    prompt.push_str("Provide a match score (0-100) and detailed gap analysis.\n\n");
    prompt.push_str("--- JOB DESCRIPTION ---\n");
    prompt.push_str(jd_text);
    prompt.push_str("\n\n--- RESUME (.tex) ---\n");
    prompt.push_str(tex_content);
    prompt.push_str("\n\n--- OUTPUT FORMAT ---\n");
    prompt.push_str("Score: <number>/100\n");
    prompt.push_str("Matched Keywords: <comma-separated list>\n");
    prompt.push_str("Missing Keywords: <comma-separated list>\n");
    prompt.push_str("Gaps:\n- <gap 1>\n- <gap 2>\n");
    prompt
}

/// Perform a basic local keyword match (no AI needed).
pub fn keyword_match(tex_content: &str, keywords: &[String]) -> ScoreResult {
    let lower_tex = tex_content.to_lowercase();
    let mut matched = Vec::new();
    let mut missing = Vec::new();

    for keyword in keywords {
        if lower_tex.contains(&keyword.to_lowercase()) {
            matched.push(keyword.clone());
        } else {
            missing.push(keyword.clone());
        }
    }

    let total = keywords.len().max(1);
    let match_rate = matched.len() as f64 / total as f64 * 100.0;

    ScoreResult {
        overall_score: match_rate,
        keyword_match_rate: match_rate,
        matched_keywords: matched,
        missing_keywords: missing,
        gap_analysis: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_match_all_found() {
        let tex = "Experience with Rust, Python, and Docker containers";
        let keywords = vec!["Rust".to_string(), "Python".to_string(), "Docker".to_string()];
        let result = keyword_match(tex, &keywords);
        assert_eq!(result.overall_score, 100.0);
        assert!(result.missing_keywords.is_empty());
    }

    #[test]
    fn test_keyword_match_partial() {
        let tex = "Experience with Rust development";
        let keywords = vec!["Rust".to_string(), "Python".to_string()];
        let result = keyword_match(tex, &keywords);
        assert_eq!(result.overall_score, 50.0);
        assert_eq!(result.matched_keywords.len(), 1);
        assert_eq!(result.missing_keywords.len(), 1);
    }

    #[test]
    fn test_keyword_match_none() {
        let tex = "No relevant skills here";
        let keywords = vec!["Kubernetes".to_string(), "Terraform".to_string()];
        let result = keyword_match(tex, &keywords);
        assert_eq!(result.overall_score, 0.0);
    }

    #[test]
    fn test_keyword_match_case_insensitive() {
        let tex = "RUST programming";
        let keywords = vec!["rust".to_string()];
        let result = keyword_match(tex, &keywords);
        assert_eq!(result.overall_score, 100.0);
    }

    #[test]
    fn test_keyword_match_empty_keywords() {
        let result = keyword_match("some content", &[]);
        assert_eq!(result.overall_score, 0.0);
    }

    #[test]
    fn test_build_score_prompt_contains_content() {
        let prompt = build_score_prompt("\\section{Skills}", "Looking for Rust developer");
        assert!(prompt.contains("\\section{Skills}"));
        assert!(prompt.contains("Rust developer"));
    }
}
