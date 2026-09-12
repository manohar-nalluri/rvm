# RVM - Resume Version Manager

## Project Overview
RVM is a CLI tool built in Rust that brings git-like version control to resume management.
It provides branching, LaTeX compilation, AI-powered tailoring, and job application tracking.

## Architecture
This is a **Rust workspace** with 8 independent crates in `crates/`:

| Crate | Purpose | Key Dependencies |
|-------|---------|------------------|
| `rvm-types` | Shared types, errors, traits | serde, chrono, thiserror |
| `rvm-core` | Version control: branches, commits, diff, merge | rvm-types, similar |
| `rvm-compiler` | LaTeX-to-PDF pipeline | rvm-types |
| `rvm-validator` | Post-compilation checks | rvm-types, lopdf |
| `rvm-tracker` | Job application lifecycle | rvm-types, chrono |
| `rvm-ai` | AI integration (Claude Code skills) | rvm-types |
| `rvm-tui` | Terminal UI (ratatui) | rvm-types, rvm-core, rvm-tracker, ratatui |
| `rvm-cli` | Binary entry point (clap) | All crates |

## Dependency Graph (no circular deps)
```
rvm-types (foundation - no internal deps)
    ├── rvm-core
    ├── rvm-compiler
    ├── rvm-validator
    ├── rvm-tracker
    ├── rvm-ai
    ├── rvm-tui (also depends on rvm-core, rvm-tracker)
    └── rvm-cli (depends on all crates)
```

## Key Conventions
- **Error handling**: Use `RvmError` / `RvmResult<T>` from `rvm-types` everywhere
- **Serialization**: All persistent data uses serde JSON (commits, branches, jobs)
- **Config**: `rvm.toml` at workspace root, parsed via `toml` crate
- **Logging**: Use `tracing` crate (not `println!` in library code)
- **Tests**: Unit tests colocated in modules, integration tests in `tests/`

## Build & Test Commands
```bash
cargo build                    # Build all crates
cargo build -p rvm-cli         # Build just the CLI binary
cargo test                     # Run all tests
cargo test -p rvm-core         # Test a specific crate
cargo run -- init              # Run the rvm binary
cargo clippy --workspace       # Lint all crates
```

## File Layout Convention
Each crate follows this pattern:
```
crates/<name>/
  Cargo.toml          # Uses workspace dependencies
  src/
    lib.rs            # Public API exports
    <module>.rs       # One module per responsibility
```

## .rvm/ Directory Structure (user workspace, not this repo)
```
.rvm/
  HEAD                 # Current branch name
  config               # Repository-level settings (JSON)
  branches/
    <name>/
      branch.json      # Branch metadata
      commits.json     # Commit history
      snapshot.tex     # Latest .tex content
      job.json         # Job application metadata
  templates/           # Reusable LaTeX snippets
  skills/              # AI skill definitions (SKILL.md)
```

## Development Phases
1. **Phase 1** (Current): Core foundation - init, branch, checkout, commit, log, compilation
2. **Phase 2**: Validation suite, diff, merge with conflict resolution
3. **Phase 3**: TUI with ratatui
4. **Phase 4**: Job tracking per branch
5. **Phase 5**: AI integration via Claude Code skills
6. **Phase 6**: Polish - packaging, docs, export bundling, plugins
