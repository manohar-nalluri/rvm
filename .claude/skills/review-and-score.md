# Skill: Review & Score Resume

When the user wants to check how well their resume matches a job description, or wants a quality review.

## Score Against JD

When user says "score my resume", "how well does it match", "check keywords":

### 1. Load context
- Read the `.tex` file from workspace (auto-detected, any filename)
- Read `.rvm/branches/<current>/job.json` for the JD text
- If no JD saved, ask the user to paste one

### 2. Extract and match keywords
Extract keywords from the JD and check which appear in the resume:

**Report format:**
```
Score: 78% keyword match

Matched (14/18):
  Python, React, Node.js, AWS, Docker, CI/CD, REST APIs,
  microservices, PostgreSQL, Git, Agile, TypeScript, MongoDB, Redis

Missing (4/18):
  Kafka, Terraform, GraphQL, Datadog

Suggestions:
  - Add "Kafka" to your TechCorp bullet about event-driven architecture
  - Mention "Terraform" in your infrastructure/deployment bullet
  - "GraphQL" could fit in the API design bullet for WhisperNet
  - "Datadog" - only add if you have actual experience with it
```

### 3. Provide actionable suggestions
For each missing keyword, suggest WHERE in the resume it could naturally fit:
- Which bullet point to modify
- How to rephrase to include the term
- Flag if the keyword requires experience the user doesn't seem to have

## Quality Review

When user says "review my resume", "any issues", "check for problems":

### Check these categories:

**1. Impact & Metrics**
- Every bullet should quantify impact where possible
- Flag bullets without numbers: "Improved X" -> "Improved X by 40%"
- Suggest metrics: users served, latency reduced, team size, cost saved

**2. Action Verbs**
- Each bullet should start with a strong verb
- Flag weak starts: "Responsible for", "Helped with", "Worked on"
- Suggest: "Led", "Designed", "Implemented", "Reduced", "Scaled"

**3. ATS Compatibility**
- No images, graphics, or multi-column layouts
- Standard section headings (Experience, Education, Skills, Projects)
- No tables for layout (tabular for formatting is fine)
- Check for `\pdfgentounicode` compatibility with tectonic

**4. Content Balance**
- 3-5 bullets per role
- Skills section matches what's demonstrated in experience
- Most recent role has the most detail
- Education is concise (no high school, no coursework unless relevant)

**5. LaTeX Issues**
- Missing closing braces or environments
- `\resumeSubheading` has exactly 4 arguments
- Consistent formatting across all roles

### Output format:
```
Resume Review Summary
=====================

Strengths:
  - Strong action verbs throughout
  - Good use of metrics in TechCorp bullets

Issues Found:
  [HIGH] Bullet "Helped with deployment" - weak verb, no metrics
         Suggest: "Automated deployment pipeline, reducing release time by 60%"

  [MEDIUM] Skills section lists 12 items but only 6 appear in experience
           Consider removing skills you can't demonstrate

  [LOW] Education section could be more concise
        Remove GPA unless it's 3.7+

ATS Score: 9/10 (clean formatting, standard sections)
```

## Example Interactions

User: "Score my resume against the Google JD"
-> Extract keywords from job.json, compare, show match % and missing keywords

User: "Review my resume"
-> Full quality review: impact, verbs, ATS, content balance, LaTeX

User: "What keywords am I missing for Stripe?"
-> Checkout stripe branch, score, list missing with suggestions
