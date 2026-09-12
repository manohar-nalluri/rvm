# RVM - Resume Version Manager

## Quick Start

```bash
# 1. Initialize workspace
mkdir ~/my-resume && cd ~/my-resume
rvm init

# 2. Save your .tex file (any name: resume.tex, yourname.tex, etc), then commit
rvm commit -m "Initial resume"

# 3. Create a tailored branch for a job
rvm branch google-swe
rvm checkout google-swe
# Edit resume.tex for Google, then:
rvm commit -m "Tailored for Google SWE"

# 4. Export when ready to apply
rvm export
```

## Commands Reference

### Workspace
| Command | Description |
|---------|-------------|
| `rvm init` | Initialize `.rvm/` workspace in current directory |
| `rvm status` | Job tracking dashboard across all branches |
| `rvm tui` | Launch interactive terminal UI |

### Version Control
| Command | Description |
|---------|-------------|
| `rvm commit -m "msg"` | Snapshot .tex file + auto-compile PDF |
| `rvm log` | View commit history for current branch |
| `rvm diff` | Diff current branch against previous commit |
| `rvm diff main` | Diff current branch against main |
| `rvm diff branchA branchB` | Diff two branches |

### Branch Management
| Command | Description |
|---------|-------------|
| `rvm branch` | List active branches |
| `rvm branch <name>` | Create new branch from current |
| `rvm checkout <name>` | Switch branch (updates .tex file, recompiles PDF) |
| `rvm merge <name>` | Merge branch into current (three-way merge) |
| `rvm archive <name>` | Archive a branch (hide from active list) |
| `rvm branch --archived` | List archived branches |

### Export & Delivery
| Command | Description |
|---------|-------------|
| `rvm export` | Bundle current branch (tex, pdf, job.json) as zip |
| `rvm export <branch>` | Export a specific branch |

### AI Commands
| Command | Description |
|---------|-------------|
| `rvm ai tailor` | Tailor resume to job description |
| `rvm ai score` | Score resume against job description |
| `rvm ai cover-letter` | Generate cover letter |
| `rvm ai review` | Grammar, impact, clarity review |
| `rvm ai prep` | Interview prep from resume-JD alignment |
| `rvm ai init` | Initialize AI skill files |

### TUI Keybindings
| Key | Action |
|-----|--------|
| `h` / `l` | Navigate panels (branch list / main / details) |
| `j` / `k` | Scroll up/down |
| `:` | Open command palette |
| `q` | Quit |

**Command palette**: `log`, `diff`, `preview`, `jobs`, `checkout <branch>`, `quit`

## Typical Workflow

```
1. rvm init                          # One-time setup
2. Place resume.tex in directory
3. rvm commit -m "Base resume"       # Save + compile PDF

--- For each job application ---
4. rvm branch <company-role>         # e.g. google-swe, stripe-backend
5. rvm checkout <company-role>
6. Edit resume.tex for the role
7. rvm commit -m "Tailored for ..."  # Auto-compiles PDF
8. rvm export                        # Package for submission
9. rvm checkout main                 # Back to base

--- After application cycle ---
10. rvm archive <company-role>       # Clean up
```

## File Requirements
- Resume must be named **`resume.tex`** in the workspace root
- LaTeX must compile with tectonic (`brew install tectonic`)
- For tectonic compatibility, replace `\pdfgentounicode=1` with:
  ```latex
  \ifx\pdfgentounicode\undefined\else\pdfgentounicode=1\fi
  ```

## .rvm/ Structure
```
.rvm/
  HEAD                 # Current branch name
  config               # Settings (JSON)
  branches/
    <name>/
      branch.json      # Branch metadata
      commits.json     # Commit history
      snapshot.tex     # Latest .tex content
      job.json         # Job application metadata
  templates/           # LaTeX snippets
  skills/              # AI skill definitions
```
