/// Build the prompt for AI resume review.
pub fn build_review_prompt(tex_content: &str) -> String {
    let mut prompt = String::new();
    prompt.push_str("Perform a comprehensive review of the following resume.\n\n");
    prompt.push_str("--- RESUME (.tex) ---\n");
    prompt.push_str(tex_content);
    prompt.push_str("\n\n--- REVIEW CRITERIA ---\n");
    prompt.push_str("1. Grammar and spelling\n");
    prompt.push_str("2. Impact and clarity of bullet points\n");
    prompt.push_str("3. Quantification of achievements\n");
    prompt.push_str("4. Overall structure and flow\n");
    prompt.push_str("5. Consistency in formatting and tense\n");
    prompt.push_str("6. Action verb usage\n");
    prompt.push_str("\n--- OUTPUT FORMAT ---\n");
    prompt.push_str("Category: <name>\n");
    prompt.push_str("  - Issue: <description>\n");
    prompt.push_str("  - Suggestion: <fix>\n");
    prompt
}
