# RVM - Resume Version Manager Skill

## Overview
RVM is a CLI tool for managing resume versions with git-like branching, LaTeX compilation, and AI-powered tailoring.

## Commands
- `rvm init` - Initialize workspace
- `rvm branch <name>` - Create branch
- `rvm checkout <name>` - Switch branch
- `rvm commit -m "msg"` - Commit changes
- `rvm diff [a] [b]` - Show diff
- `rvm merge <branch>` - Merge branch
- `rvm log` - View history
- `rvm status` - Job dashboard
- `rvm ai tailor` - AI-tailor resume to JD
- `rvm ai score` - Score resume vs JD
- `rvm ai create --jd <file>` - Create branch from JD
- `rvm ai cover-letter` - Generate cover letter
- `rvm ai review` - Review resume quality

## File Formats
- `.tex` - LaTeX resume source
- `rvm.toml` - Configuration
- `branch.json` - Branch metadata
- `commits.json` - Commit history
- `job.json` - Job application metadata
