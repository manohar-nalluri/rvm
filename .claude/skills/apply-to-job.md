# Skill: Apply to Job

When the user provides a **job description** (pasted text or file) and a **company name/role**, perform the full RVM application setup:

## Steps

### 1. Create the branch
```bash
rvm branch <company>-<role>
rvm checkout <company>-<role>
```
- Branch name format: lowercase, hyphenated (e.g. `google-swe`, `stripe-backend-eng`)
- Derive from company + role. Ask user if unclear.

### 2. Save job metadata
Write `.rvm/branches/<branch>/job.json` with:
```json
{
  "company": "<Company Name>",
  "role": "<Role Title>",
  "jd_text": "<full JD text>",
  "jd_url": "<URL if provided>",
  "status": "draft",
  "applied_date": null,
  "platform": "<where the user found it: linkedin, indeed, company site, referral, etc>",
  "contacts": [],
  "events": [],
  "notes": null
}
```
Ask the user which platform they found the job on (LinkedIn, Indeed, company website, referral, etc).

### 3. Extract keywords and tailor the resume
- Read the current `.tex` file in the workspace
- Extract keywords from the JD (technical skills, tools, frameworks, methodologies)
- Identify which keywords are **missing** from the current resume
- Modify the `.tex` file to naturally incorporate missing keywords:
  - Rewrite bullet points to include JD-specific terms
  - Reorder skills section to lead with the most relevant
  - Add relevant technologies/tools mentioned in the JD
  - Keep experience truthful - only rephrase, don't fabricate
- Preserve all LaTeX formatting and structure
- Keep to 1 page

### 4. Commit the tailored version
```bash
rvm commit -m "Tailored for <Company> <Role>"
```

### 5. Show a summary
Print:
- Branch name created
- Keywords added to resume
- Keywords still missing (if any couldn't be naturally incorporated)
- Diff preview: `rvm diff main`
- Next steps: "Edit further if needed, then `rvm export` when ready to submit"

## Example Usage

User: "I want to apply to this Google SWE position" + pastes JD

Result:
1. Creates branch `google-swe`
2. Saves JD to job.json with company="Google", role="Software Engineer"
3. Tailors resume with Google-specific keywords (distributed systems, Go, gRPC, etc)
4. Commits as "Tailored for Google Software Engineer"
5. Shows diff of what changed

## Important Rules
- NEVER fabricate experience or skills the user doesn't have
- Only rephrase existing experience to better match JD terminology
- If a keyword requires experience the user doesn't have, list it as "missing" in the summary
- Always preserve the LaTeX structure and compilability
- The `.tex` file can be any name (not just resume.tex) - use `rvm` commands which auto-detect it
