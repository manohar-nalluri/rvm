---
description: Tailor resume to a job description using RVM - create branch, extract keywords, rewrite content human-style, ensure ATS compatibility, fit single page
---

Load the `rvm` skill first for full context on RVM architecture, LaTeX template, tailoring rules, and ATS guidelines. Then follow the workflow below.

You are an expert resume tailor. Follow the RVM workflow to create a job-tailored resume branch.

## Step 1: Detect RVM workspace

Check if `.rvm/` exists in the current directory. If not, ask the user if you should run `rvm init` first.

Use this binary for all rvm commands (or `rvm` if on PATH):
```bash
RVM="${RVM:-rvm}"
```

## Step 2: Parse the JD

From the job description provided (or ask for it), extract:
- Company name
- Role title
- Branch name: `<company>-<role>` lowercase, hyphenated (e.g. `amazon-sde`, `google-swe`)
- Keywords: technical skills, tools, frameworks, methodologies
- Requirements and responsibilities

## Step 3: Create branch and save JD

```bash
$RVM branch <branch-name>
$RVM checkout <branch-name>
```

Write `.rvm/branches/<branch>/job.json` with company, role, jd_text, status="draft".

## Step 4: Tailor the resume

Read the `.tex` file in the **project root** (the working file). Rewrite it to match the JD with these rules:

### CRITICAL: Edit the working file, NOT the snapshot
- Edit the `.tex` file in the project root directory
- DO NOT edit `.rvm/branches/<name>/snapshot.tex`
- After editing, run `$RVM commit` which reads the working file and updates the snapshot automatically

### CRITICAL: Sound human, not AI
- NEVER use: "spearheaded", "orchestrated", "leveraged", "transformative", "game-changing", "passionate about", "exceptional", "outstanding"
- Write like a competent engineer describing their work, not a marketing brochure
- Keep it truthful - reframe existing experience in JD language, don't invent new experience

### ATS optimization
- Use EXACT keywords from the JD (ATS does string matching)
- Standard section headings: Objective, Experience, Projects, Skills, Education
- Single column, no tables for layout

### Single page, no bottom gap
- Must fit exactly one page with no wasted space at bottom
- Adjust by content only: trim less relevant bullets, condense descriptions
- Use `\addtolength{\textheight}{1.4in}` and similar to fill the page

### Tailoring strategy
- **Objective**: 2-3 sentences directly addressing the role with 3-5 JD terms
- **Experience**: Rewrite bullets to include JD keywords, lead with most relevant, 4-6 bullets per role
- **Projects**: Reorder by relevance, highlight JD technologies, trim to 1-2 if tight
- **Skills**: Reorder to match JD priorities, use exact JD terminology
- **Education**: Keep as-is

### NEVER fabricate
- Don't invent experience, skills, companies, or metrics
- Only rephrase what the user actually has
- Report keywords that couldn't be naturally incorporated

## Step 5: Commit and show results

```bash
$RVM commit -m "Tailored for <Company> <Role>"
$RVM diff main
```

Show summary: branch name, keywords added, keywords missing, diff, compilation status, next steps.

Show summary: branch name, keywords added, keywords missing, diff, compilation status, next steps.

---

If no JD was provided with the command, ask the user to paste the job description or provide a file path.
