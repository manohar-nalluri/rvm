# Building Resumes with RVM

## Resume LaTeX Template

RVM expects a `resume.tex` file using standard LaTeX. Here's a minimal ATS-friendly template:

```latex
\documentclass[letterpaper,11pt]{article}
\usepackage[empty]{fullpage}
\usepackage{titlesec}
\usepackage[usenames,dvipsnames]{color}
\usepackage{enumitem}
\usepackage[hidelinks]{hyperref}
\usepackage{fancyhdr}
\usepackage{fontawesome5}

\pagestyle{fancy}
\fancyhf{}
\renewcommand{\headrulewidth}{0pt}
\addtolength{\oddsidemargin}{-0.5in}
\addtolength{\textwidth}{1in}
\addtolength{\topmargin}{-0.7in}
\addtolength{\textheight}{1.0in}

% Tectonic-compatible (replaces \pdfgentounicode=1)
\ifx\pdfgentounicode\undefined\else\pdfgentounicode=1\fi

\titleformat{\section}{\vspace{-5pt}\scshape\raggedright\large}{}{0em}{}[\titlerule\vspace{-5pt}]

\begin{document}

\begin{center}
  \textbf{\Huge Your Name} \\
  \small \href{mailto:email@example.com}{email@example.com} $|$
  \href{https://github.com/you}{GitHub} $|$
  \href{https://linkedin.com/in/you}{LinkedIn}
\end{center}

\section{Experience}
% Use \resumeSubheading with 4 arguments: {title}{dates}{subtitle}{location}

\section{Projects}

\section{Skills}
\small{
\textbf{Languages:} Python, Rust, JavaScript \\
\textbf{Tools:} Git, Docker, AWS
}

\section{Education}

\end{document}
```

## Tailoring Workflow

```bash
# Start from your base resume
rvm checkout main

# Create a branch per job application
rvm branch stripe-backend-2026
rvm checkout stripe-backend-2026

# Tailor: edit resume.tex to match the JD
# Focus on: keywords, relevant projects, reordered skills

# Commit and compile
rvm commit -m "Stripe Backend Eng - emphasized distributed systems"

# Review diff against base
rvm diff main

# Export for submission
rvm export
```

## ATS Tips for LaTeX Resumes

1. **No images, graphics, or tables for layout** - ATS can't parse them
2. **Use standard section headings**: Experience, Education, Skills, Projects
3. **No multi-column layouts** - confuses ATS parsers
4. **Keep to 1 page** - unless 10+ years experience
5. **Include exact keywords** from the job description
6. **Quantify achievements**: "Reduced latency by 40%" not "Improved performance"
7. **Use `\item` bullets** under each role (3-5 per position)

## Common LaTeX Issues with Tectonic

| Issue | Fix |
|-------|-----|
| `\pdfgentounicode` undefined | Use `\ifx\pdfgentounicode\undefined\else\pdfgentounicode=1\fi` |
| Missing 4th arg on `\resumeSubheading` | Always pass 4 args: `{title}{dates}{subtitle}{}` |
| FontAwesome warnings | Cosmetic - safe to ignore |
| Package not found | Tectonic auto-downloads on first compile (needs internet) |

## Branch Naming Convention

Use `<company>-<role>` format:
```
main              # Base resume (never delete)
google-swe        # Google Software Engineer
stripe-backend    # Stripe Backend Engineer
meta-frontend     # Meta Frontend Engineer
```

## Merge Strategy

After getting an offer or finishing a cycle, merge improvements back:
```bash
rvm checkout main
rvm merge google-swe        # Pull in any general improvements
rvm archive google-swe      # Clean up
```
