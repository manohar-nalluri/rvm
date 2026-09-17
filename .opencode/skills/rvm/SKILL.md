---
name: rvm
description: Tailor LaTeX resumes to job descriptions using RVM - create branches, extract JD keywords, rewrite content to sound human (not AI), ensure ATS compatibility, and fit tightly on a single page with no wasted space
---

# RVM - Resume Version Manager & JD Tailoring

## Overview

RVM is a git-like version control system for LaTeX resumes. When the user invokes `/rvm`, you are an expert resume tailor that:

1. **Detects** if the current workspace has an `.rvm/` directory
2. **Initializes** RVM if needed
3. **Creates branches** from job descriptions
4. **Tailors resumes** to match JDs with human-sounding, ATS-optimized content
5. **Commits** changes and compiles to PDF

Your goal: produce resumes that pass ATS screening, showcase the user's skills honestly (not selling or boasting), sound human-written, and fit on a single page by adjusting content only - never by changing margins, font sizes, or spacing.

## Detection & Initialization

### Step 1: Check for .rvm directory

When the user invokes `/rvm` with a JD, first check:

```bash
# Check if we're in an RVM workspace
ls -d .rvm 2>/dev/null
```

### Step 2: If no .rvm exists, offer to initialize

If there's no `.rvm/` directory, ask the user:

> "This directory doesn't have an RVM workspace yet. I can initialize one here with `rvm init`. Should I proceed?"

If yes, run:
```bash
rvm init
```

This creates:
- `.rvm/HEAD` - current branch pointer
- `.rvm/config` - workspace settings
- `.rvm/branches/main/` - base branch with initial resume
- `rvm.toml` - compiler and validation config

### Step 3: Verify the .tex file exists

After init (or if already initialized), confirm the resume `.tex` file is present:
```bash
ls *.tex
```

If missing after init, the user needs to provide their base resume `.tex` file.

## Branch Creation & JD Processing

### Step 1: Parse the JD

When the user provides a JD (pasted text, file path, or URL), extract:

1. **Company name** - e.g. "Amazon", "Google", "Stripe"
2. **Role title** - e.g. "Software Development Engineer", "Backend Engineer"
3. **Branch name** - derive as `<company>-<role>` lowercase, hyphenated
   - Examples: `amazon-sde`, `google-swe`, `stripe-backend-eng`, `meta-frontend`
4. **Keywords** - technical skills, tools, frameworks, methodologies, concepts
5. **Requirements** - must-have vs nice-to-have qualifications
6. **Responsibilities** - what the role actually does day-to-day

### Step 2: Create the branch

```bash
rvm branch <branch-name>
rvm checkout <branch-name>
```

This forks from the current branch (usually `main`), copying the snapshot and commit history.

### Step 3: Save job metadata

Write `.rvm/branches/<branch>/job.json`:

```json
{
  "company": "<Company Name>",
  "role": "<Role Title>",
  "jd_text": "<full JD text>",
  "jd_url": "<URL if provided>",
  "status": "draft",
  "applied_date": null,
  "platform": "<linkedin|indeed|company-site|referral|other>",
  "contacts": [],
  "events": [],
  "notes": null
}
```

Ask the user which platform they found the job on if unclear.

## Resume Tailoring Rules

### Core Principles

1. **NEVER fabricate experience or skills** - only rephrase what the user actually has
2. **Showcase, don't sell** - state what you did and what it involved, don't hype it up. Recruiters want facts, not a pitch.
3. **Sound human, not AI** - avoid these AI tells:
   - "Spearheaded", "orchestrated", "leveraged", "spearheading"
   - "Transformative", "game-changing", "cutting-edge" (unless in JD)
   - Overly enthusiastic adjectives: "exceptional", "outstanding", "remarkable"
   - Generic filler: "passionate about", "enthusiastic", "driven professional"
   - Bullet points that all start the same way
   - Numbers that seem made up (use ~ or "approximately" if approximate)
4. **Match JD terminology** - if JD says "microservices", use "microservices" not "distributed services"
5. **Keep it truthful** - reframe existing experience in JD's language, don't invent new experience
6. **Single page, tightly packed** - adjust by content only, never by margins, fonts, or spacing

### Tailoring Strategy

#### 1. Objective Section
- Rewrite to directly address the role and company
- Include 3-5 key JD terms naturally
- Keep it to 2-3 sentences
- State what you've done and what you're looking for - don't sell or boast

**Bad (selling/AI):** "Passionate software engineer with exceptional expertise in leveraging cutting-edge technologies to deliver transformative solutions."

**Bad (boastful):** "Top-performing engineer who consistently delivers outstanding results in fast-paced environments."

**Good (showcase):** "Software engineer who's spent the last year building production systems in Python and Node.js. I've worked with distributed services, REST APIs, and real-time data pipelines. Looking to bring that experience to a team that ships code fast and cares about correctness."

#### 2. Experience Section
- Rewrite bullet points to incorporate JD keywords
- Keep the same facts, change the framing
- Lead with the most relevant bullets for this specific role
- Use action verbs that match the JD's tone
- Include metrics when available, but don't invent them
- 4-6 bullets per role (trim less relevant ones if space is tight)

**Bad (selling/AI):** "Spearheaded the development of a revolutionary microservices architecture that orchestrated seamless data flow across distributed systems, resulting in a 47% improvement in operational efficiency."

**Bad (boastful):** "Consistently delivered outstanding results, exceeding all performance targets and earning recognition as the team's top contributor."

**Good (showcase):** "Built a microservices-based order processing system in Node.js handling 10k+ requests/day. Designed the API contracts, wrote the core matching logic, and set up monitoring so we'd know before customers did when something broke."

#### 3. Projects Section
- Reorder to put most relevant projects first
- Rewrite descriptions to highlight JD-relevant technologies
- Trim to 1-2 projects if space is tight
- Keep tech stack line accurate

#### 4. Skills Section
- Reorder categories to match JD priorities
- Add skills the user actually has that are mentioned in JD
- Remove skills that aren't relevant to this specific role (if space is tight)
- Use exact JD terminology: if JD says "CI/CD pipelines", use that not "continuous integration"

#### 5. Education Section
- Keep as-is unless JD has specific education requirements to highlight
- Can add relevant coursework if JD mentions specific CS fundamentals

### Space Management (Single Page, No Bottom Gap)

The resume MUST fit on exactly one page. **NEVER change margins, font sizes, or `\vspace` values.** Only adjust by content:

1. **Remove least relevant bullets** - if a bullet point doesn't support the JD's requirements, cut it first
2. **Add relevant bullets** - if there's unused space and the user has relevant experience not yet mentioned, add a concise bullet
3. **Trim project descriptions** - reduce to 1-2 bullets per project if space is tight, expand to 3 if space allows
4. **Remove Objective section** - only if space is extremely tight and content is more valuable elsewhere; it's optional
5. **Rewrite bullets to be concise** - remove filler words, keep the substance
6. **Check with `rvm commit`** - auto-compile will show if it's over 1 page; adjust content, not formatting

### ATS Optimization

1. **No tables for layout** - the resume template already avoids this
2. **Standard section headings**: Objective, Experience, Projects, Skills, Education
3. **Exact keyword matching** - ATS does string matching, use the exact terms from JD
4. **No images, graphics, or icons** in content area (header icons are fine)
5. **Single column layout** - already the case with this template
6. **Include acronyms AND full forms** on first use: "Software Development Kit (SDK)"
7. **Avoid headers/footers with critical info** - contact info should be in body

## Execution Workflow

When the user provides a JD, execute this full workflow:

```bash
# Use the rvm binary (from PATH or built target):
RVM="${RVM:-rvm}"

# 1. Detect/init (if needed)
# Check for .rvm/, offer init if missing

# 2. Create branch
$RVM branch <company>-<role>
$RVM checkout <company>-<role>

# 3. Read the current .tex file in the ROOT directory (the working file)
# This is the file that rvm checkout restores - edit THIS file, NOT the snapshot
cat <resume>.tex

# 4. Edit the .tex file in the ROOT directory with tailored content
# Apply all tailoring rules above
# IMPORTANT: Edit the working .tex file in the project root, NOT .rvm/branches/<name>/snapshot.tex
# rvm commit will read the working file and update the snapshot automatically

# 5. Commit (auto-compiles if configured)
$RVM commit -m "Tailored for <Company> <Role>"

# 6. Show diff
$RVM diff main

# 7. Show validation results (from commit output)
```

## Keyword Extraction & Matching

### Extract from JD
1. **Technical skills**: languages, frameworks, tools, platforms
2. **Concepts**: methodologies, architectures, patterns
3. **Soft skills**: only if emphasized heavily in JD
4. **Requirements**: degrees, years of experience, specific technologies

### Match against user's resume
1. Read the `main` branch snapshot to understand the user's actual experience
2. Cross-reference JD keywords with user's skills
3. **Add** keywords the user genuinely has
4. **Add adjacent skills confidently** - if the user knows something similar, add the JD keyword as if they already know it. Example: knows Node.js → add Express/NestJS; knows Python → add Django/FastAPI; knows SQL → add PostgreSQL/MySQL. State it as fact, don't hedge or say "familiar with" or "willing to learn"
5. **Reframe** existing bullets to include JD terminology
6. **Only skip** keywords that are entirely unrelated to the user's background (e.g., adding iOS/Swift when they've never done mobile)
7. **Report** keywords that couldn't be incorporated

### Example keyword mapping

JD says: "distributed systems, microservices, REST APIs, Python, AWS, Docker, CI/CD"
User has: "Python, Node.js, AWS, Docker, REST APIs, WebSocket streams"

Action:
- Emphasize "REST APIs" in experience bullets
- Add "AWS" and "Docker" prominently in skills
- If user has worked with multiple services, reframe as "distributed systems" or "microservices" (truthful reframing)
- Add "CI/CD" if user has used any deployment pipelines (they likely have, even if not named that)
- Only skip if truly unrelated to anything they've done

## Post-Tailoring Summary

After completing the workflow, always show:

1. **Branch created**: `<branch-name>`
2. **Keywords added**: list of JD keywords incorporated
3. **Keywords missing**: JD keywords that couldn't be naturally added (with explanation)
4. **Diff preview**: `rvm diff main` output
5. **Compilation status**: success/failure from auto-compile
6. **Next steps**:
   - "Edit `resume.tex` further if needed"
   - "Run `rvm commit -m 'additional changes'` after edits"
   - "Run `rvm export` when ready to submit PDF"
   - "Run `rvm ai score` to check ATS compatibility"

## LaTeX Template Reference

The user's resume uses this template structure:

```latex
\documentclass[letterpaper,11pt]{article}
\usepackage[empty]{fullpage}
\usepackage{titlesec}
\usepackage[usenames,dvipsnames]{color}
\usepackage{enumitem}
\usepackage[hidelinks]{hyperref}
\usepackage{fancyhdr}
\usepackage{fontawesome5}

% Page margins (adjust for single-page fit)
\addtolength{\oddsidemargin}{-0.5in}
\addtolength{\textwidth}{1in}
\addtolength{\topmargin}{-0.85in}
\addtolength{\textheight}{1.4in}

% Section formatting
\titleformat{\section}{
  \vspace{-6pt}\scshape\raggedright\large
}{}{0em}{}[\color{black}\titlerule \vspace{-6pt}]

% Custom commands
\newcommand{\resumeItem}[1]{\item\small{{#1 \vspace{-2pt}}}}
\newcommand{\resumeSubheading}[4]{\vspace{-2pt}\item
  \begin{tabular*}{0.97\textwidth}[t]{l@{\extracolsep{\fill}}r}
    \textbf{#1} & #2 \\
    \textit{\small#3} & \textit{\small #4} \\
  \end{tabular*}\vspace{-7pt}}
\newcommand{\resumeSubHeadingListStart}{\begin{itemize}[leftmargin=0.15in, label={}]}
\newcommand{\resumeSubHeadingListEnd}{\end{itemize}}
\newcommand{\resumeItemListStart}{\begin{itemize}[leftmargin=0.15in]}
\newcommand{\resumeItemListEnd}{\end{itemize}\vspace{-5pt}}

\begin{document}

% Header (centered, with contact info)
\begin{center}
    \textbf{\Huge \scshape Full Name} \\ \vspace{6pt}
    \small
    \faIcon{github} \href{https://github.com/username}{github.com/username} $|$
    \faIcon{linkedin} \href{https://linkedin.com/in/username}{linkedin} $|$
    \faIcon{envelope} \href{mailto:email@example.com}{email} $|$
    \faIcon{phone} +XX XXXXX XXXXX
\end{center}

% Sections: Objective, Experience, Projects, Skills, Education
\section{Objective}
\small{...}

\section{Experience}
\resumeSubHeadingListStart
\resumeSubheading
{Company}{Dates}{Role (tech stack)}{}
\resumeItemListStart
  \resumeItem{Bullet point with \textbf{keywords}}
\resumeItemListEnd
\resumeSubHeadingListEnd

\section{Projects}
\resumeSubHeadingListStart
\resumeSubheading
{\textbf{ProjectName}}{}
{\footnotesize\emph{Tech1, Tech2, Tech3}}{}
\resumeItemListStart
  \resumeItem{Description}
\resumeItemListEnd
\resumeSubHeadingListEnd

\section{Skills}
\small{
\textbf{Languages:} ... \\
\textbf{Frameworks:} ... \\
\textbf{Concepts:} ... \\
\textbf{Tools:} ...
}

\section{Education}
\resumeSubHeadingListStart
  \resumeSubheading
    {University}{Years}{Degree}{}
\resumeSubHeadingListEnd

\end{document}
```

## Important Rules

1. **TRUTHFULNESS**: Never invent experience, skills, companies, or metrics
2. **SHOWCASE, DON'T SELL**: State what you did and what technologies you used. Don't hype, boast, or pitch. Let the facts speak.
3. **HUMAN VOICE**: Write like a competent engineer describing their work, not a marketing brochure
4. **JD ALIGNMENT**: Every change should trace back to something in the JD
5. **SINGLE PAGE**: Non-negotiable - adjust by content only (add/remove bullets, trim/expand descriptions)
6. **NO FORMATTING CHANGES**: Never change margins, font sizes, `\vspace`, or `\titleformat` values to fit the page
7. **ATS COMPATIBLE**: Standard headings, no tables for layout, exact keyword matching
8. **PRESERVE STRUCTURE**: Keep the LaTeX template intact, only modify content
9. **COMMIT OFTEN**: Each meaningful change should be a separate commit
10. **SHOW DIFF**: Always show what changed vs the base branch
11. **REPORT GAPS**: Be honest about JD keywords that couldn't be incorporated

## RVM Commands Reference

| Command | Purpose |
|---------|---------|
| `rvm init` | Initialize RVM workspace |
| `rvm branch <name>` | Create new branch from current |
| `rvm branch --list` | List all branches |
| `rvm checkout <name>` | Switch to branch + auto-compile |
| `rvm commit -m "msg"` | Snapshot + auto-compile + validate |
| `rvm diff [branch]` | Show diff vs another branch |
| `rvm log` | View commit history |
| `rvm export` | Export branch files + PDF |
| `rvm status` | Job application dashboard |
| `rvm merge <branch>` | Merge changes from another branch |
| `rvm archive <branch>` | Soft-delete a branch |
| `rvm branch --protect <name>` | Require the system password to modify a branch |
| `rvm branch --unprotect <name>` | Remove protection (requires the system password) |

### Protected branches

`main` is protected by default. `rvm commit`, `rvm merge` into it,
`--protect`/`--unprotect`, and `rvm archive` all refuse until the operator
types their system password at an interactive prompt.

**Never attempt to write to `main`.** Always create a branch first
(`rvm branch <company>-<role>`, then `rvm checkout <company>-<role>`) and do all
work there. A refused operation exits non-zero with a "branch ... is protected"
message and writes nothing, so a failure is safe but pointless to retry.
`rvm branch --list` marks protected branches with `[protected]`.

There is deliberately no flag, environment variable, or non-interactive mode to
bypass protection, and no terminal is available to you. Do not search for a
workaround and do not edit `.rvm/config` or `rvm.toml` to change protection —
report the limitation to the operator instead.

## File Structure

```
.rvm/
  HEAD                          # Current branch name
  config                        # Workspace config (JSON)
  branches/
    <name>/
      branch.json               # Branch metadata
      commits.json              # Commit history
      snapshot.tex              # Latest .tex content
      job.json                  # Job application metadata
  skills/
    SKILL.md                    # This file
rvm.toml                        # Compiler & validation config
<resume>.tex                    # Working resume file
```
