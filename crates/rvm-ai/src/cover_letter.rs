/// Build the prompt for AI cover letter generation.
pub fn build_cover_letter_prompt(
    tex_content: &str,
    jd_text: &str,
    company: Option<&str>,
    role: Option<&str>,
) -> String {
    let mut prompt = String::new();
    prompt.push_str("Generate a professional cover letter based on the following resume and job description.\n\n");

    if let Some(company) = company {
        prompt.push_str(&format!("Company: {}\n", company));
    }
    if let Some(role) = role {
        prompt.push_str(&format!("Position: {}\n", role));
    }

    prompt.push_str("\n--- RESUME (.tex) ---\n");
    prompt.push_str(tex_content);
    prompt.push_str("\n\n--- JOB DESCRIPTION ---\n");
    prompt.push_str(jd_text);
    prompt.push_str("\n\n--- INSTRUCTIONS ---\n");
    prompt.push_str("1. Write a compelling cover letter (3-4 paragraphs)\n");
    prompt.push_str("2. Highlight the most relevant experience for this role\n");
    prompt.push_str("3. Reference specific requirements from the JD\n");
    prompt.push_str("4. Keep a professional but personable tone\n");
    prompt.push_str("5. Output as LaTeX format\n");
    prompt
}
