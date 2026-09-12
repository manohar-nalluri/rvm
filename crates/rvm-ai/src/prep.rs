/// Build the prompt for AI interview preparation.
pub fn build_prep_prompt(tex_content: &str, jd_text: &str) -> String {
    let mut prompt = String::new();
    prompt.push_str("Generate interview preparation materials based on the resume-JD alignment.\n\n");
    prompt.push_str("--- RESUME (.tex) ---\n");
    prompt.push_str(tex_content);
    prompt.push_str("\n\n--- JOB DESCRIPTION ---\n");
    prompt.push_str(jd_text);
    prompt.push_str("\n\n--- GENERATE ---\n");
    prompt.push_str("1. Likely behavioral questions (5-7)\n");
    prompt.push_str("2. Technical questions based on required skills (5-7)\n");
    prompt.push_str("3. Talking points for each major resume entry\n");
    prompt.push_str("4. Questions to ask the interviewer (3-5)\n");
    prompt.push_str("5. Potential weaknesses/gaps to prepare for\n");
    prompt
}
