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

## Branch Protection

Protected branches refuse every mutating operation until the operator
authenticates with their **system password**. This exists so an AI agent
driving `rvm` cannot damage the base resume.

- `main` is protected by default, including in workspaces created before this
  feature existed (`ProtectionConfig` defaults to `["main"]`).
- Gated operations: `commit`, `merge` into the branch, `--protect`,
  `--unprotect`, `archive`, `delete`. Creating and working on *other* branches
  is never gated.
- **Fail closed**: `list_protected` unions `rvm.toml` and `.rvm/config`, and
  each falls back to `["main"]` when missing. Deleting or clearing one config
  file cannot unlock a branch.
- No bypass exists by design: no `--force`, no environment variable, no
  non-interactive mode, since an agent could reach any of them.
- Enforcement is centralised in `rvm-core::guard::ensure_mutable`; new mutating
  operations must call it.
- Authentication (`rvm-core::auth`) runs `dscl . -authonly <user>`, letting
  `dscl` prompt with echo disabled, so the password never enters RVM's argv,
  memory, or environment, and no sudo timestamp is granted. Requires a TTY, so
  a non-interactive agent is refused outright.

Known limitation: this gates *commands*. An agent that rewrites both
`.rvm/config` and `rvm.toml` directly can still unprotect a branch. Closing
that requires sealing branch data with a keyed HMAC (see the test
`test_hand_editing_both_config_files_can_unprotect_main`).

## Recovering Past Revisions

Every commit stores its **complete** `snapshot_tex` inline in `commits.json`,
so any past revision can be recovered byte-exactly. Two commands expose that:

- `rvm show <ref>` — print a past revision's `.tex` to **stdout**; metadata goes
  to stderr so `rvm show <ref> > old.tex` works cleanly. Accepts `HEAD`, a full
  hash, an unambiguous prefix (>= 4 hex chars), or a branch name.
- `rvm restore <ref>` — write that revision back to the working file and record
  it as a **new** commit. History is never rewritten, so a restore can itself be
  undone by restoring again. It is a mutation, so branch protection gates it.

`rvm diff <ref> [<ref>]` resolves commits too. It previously accepted only
branch names despite documenting "branch/commit", so passing a hash silently
compared *empty* content and printed the entire file as added lines while still
exiting 0. An unresolvable reference is now a hard error.

Reference resolution lives in `rvm_core::commit::resolve`; `restore` reuses the
unguarded `write_commit` path so the operator is prompted for a password once,
not twice.

## ATS and Resume Quality

`docs/ATS-GUIDE.md` is the reference for how applicant tracking systems read a
resume: text-layer extraction, reading order, icon-font and ligature traps,
bullet construction, keyword strategy, one-page layout, and the exact
`pdftotext` checks to run before submitting.

`rvm-validator` is a compile-time sanity check, **not** an ATS check. It never
inspects the text layer, reading order, or keyword relevance, and it currently
has three defects (a `\begin{tabular*}` false positive, a literal `contact`
string check, and a bullet check that misses `\resumeItem` templates). See the
appendix of the guide before trusting a clean run.

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
