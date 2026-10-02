//! AI-backed operations.
//!
//! Only `tailor` is implemented against a real backend. It drives an external
//! agent CLI (see `rvm_ai::llm`) and is deliberately non-interactive: it never
//! writes to the branch it forked from, so the protected base resume cannot be
//! damaged by an automated run.

use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use serde_json::json;

use rvm_ai::llm::CliClient;
use rvm_ai::tailor::{self, Document, TailorRequest};
use rvm_core::{branch, commit, SystemAuthenticator, Workspace};
use rvm_types::{JobMetadata, RvmConfig};

/// Number of user-visible progress steps in a tailoring run.
const STEPS: u32 = 5;

#[derive(Subcommand)]
pub enum AiCommands {
    /// Tailor the resume to a job description and commit it on a new branch
    Tailor(TailorArgs),

    /// Alias for `tailor`, kept for the documented `ai create --jd` workflow
    Create(TailorArgs),

    /// Score current resume against its job description
    Score,

    /// Generate a cover letter using branch context
    CoverLetter,

    /// Generate interview prep based on resume-JD alignment
    Prep,

    /// Comprehensive resume review: grammar, impact, clarity
    Review,

    /// Initialize AI skill files in the workspace
    Init,
}

#[derive(Args, Debug, Clone)]
pub struct TailorArgs {
    /// File containing the job description
    #[arg(long, value_name = "PATH", conflicts_with = "jd_text")]
    pub jd: Option<PathBuf>,

    /// Job description as literal text
    #[arg(long, value_name = "TEXT")]
    pub jd_text: Option<String>,

    /// Hiring company (used for the branch name and stored with the branch)
    #[arg(long)]
    pub company: Option<String>,

    /// Role title (used for the branch name and stored with the branch)
    #[arg(long)]
    pub role: Option<String>,

    /// Branch to create. Defaults to a slug of company + role.
    #[arg(long)]
    pub branch: Option<String>,

    /// Branch to fork from. This branch is only read, never written.
    #[arg(long, default_value = "main")]
    pub base: String,

    /// Model id passed to the provider
    #[arg(long)]
    pub model: Option<String>,

    /// Provider dialect (antigravity_cli)
    #[arg(long)]
    pub provider: Option<String>,

    /// Seconds to wait for one model call
    #[arg(long)]
    pub timeout_seconds: Option<u64>,

    /// Skip LaTeX compilation and page validation
    #[arg(long)]
    pub no_compile: bool,

    /// Leave the workspace on its current branch instead of switching to the new one
    #[arg(long)]
    pub no_checkout: bool,

    /// Replace an existing branch of the same name
    #[arg(long)]
    pub force: bool,

    /// Print a single JSON object on stdout; progress stays on stderr
    #[arg(long)]
    pub json: bool,
}

pub fn execute(cmd: AiCommands) -> anyhow::Result<()> {
    match cmd {
        AiCommands::Tailor(args) | AiCommands::Create(args) => tailor_command(&args),
        AiCommands::Score => score(),
        AiCommands::CoverLetter => {
            println!("Generating cover letter...");
            println!("(AI integration requires Claude Code runtime)");
            Ok(())
        }
        AiCommands::Prep => {
            println!("Generating interview prep...");
            println!("(AI integration requires Claude Code runtime)");
            Ok(())
        }
        AiCommands::Review => {
            println!("Reviewing resume...");
            println!("(AI integration requires Claude Code runtime)");
            Ok(())
        }
        AiCommands::Init => {
            let cwd = std::env::current_dir()?;
            let ws = Workspace::discover(&cwd)?;
            let skills_dir = ws.skills_dir();
            let manager = rvm_ai::skill::SkillManager::new(&skills_dir);
            manager.init_default_skills()?;
            println!("Initialized AI skills in {}", skills_dir.display());
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// tailor
// ---------------------------------------------------------------------------

struct TailorReport {
    branch: String,
    base: String,
    commit: String,
    pdf: Option<PathBuf>,
    docx: Option<PathBuf>,
    pages: Option<usize>,
    keywords: Vec<String>,
    attempts: u32,
    compiled: bool,
    checked_out: bool,
}

/// How far we got, so a failure can be reported honestly in `--json` mode.
#[derive(Default)]
struct TailorTrace {
    attempts: u32,
    compiled: bool,
}

struct Accepted {
    tex: String,
    pdf: Option<PathBuf>,
    docx: Option<PathBuf>,
    pages: Option<usize>,
}

fn tailor_command(args: &TailorArgs) -> anyhow::Result<()> {
    let mut trace = TailorTrace::default();
    match run_tailor(args, &mut trace) {
        Ok(report) => {
            if args.json {
                println!(
                    "{}",
                    serde_json::to_string(&json!({
                        "ok": true,
                        "branch": report.branch,
                        "base": report.base,
                        "commit": report.commit,
                        "pdf": report.pdf.as_ref().map(|p| p.display().to_string()),
                        "docx": report.docx.as_ref().map(|p| p.display().to_string()),
                        "pages": report.pages,
                        "keywords": report.keywords,
                        "attempts": report.attempts,
                        "compiled": report.compiled,
                        "checked_out": report.checked_out,
                    }))?
                );
            } else {
                println!("Tailored branch: {}", report.branch);
                if let Some(pdf) = &report.pdf {
                    println!("PDF: {}", pdf.display());
                }
                if let Some(docx) = &report.docx {
                    println!("DOCX: {}", docx.display());
                }
            }
            Ok(())
        }
        Err(err) => {
            if args.json {
                println!(
                    "{}",
                    serde_json::to_string(&json!({
                        "ok": false,
                        "branch": null,
                        "error": err.to_string(),
                        "attempts": trace.attempts,
                        "compiled": trace.compiled,
                    }))?
                );
                std::process::exit(1);
            }
            Err(err)
        }
    }
}

fn run_tailor(args: &TailorArgs, trace: &mut TailorTrace) -> anyhow::Result<TailorReport> {
    let cwd = std::env::current_dir()?;
    let ws = Workspace::discover(&cwd)?;
    let config = ws.load_config()?;
    let ai = &config.ai;

    let provider = args.provider.clone().unwrap_or_else(|| ai.provider.clone());
    let model = args.model.clone().unwrap_or_else(|| ai.model.clone());
    let timeout = args.timeout_seconds.unwrap_or(ai.timeout_seconds);
    let binary = CliClient::resolve_binary(&ai.cli_path);

    // --- 1. read the branch we fork from ------------------------------------
    if !branch::exists(&ws, &args.base) {
        anyhow::bail!(
            "Base branch '{}' does not exist. Pass --base <branch>.",
            args.base
        );
    }
    let source_tex = read_branch_tex(&ws, &args.base)?;
    step(
        1,
        format!("Base '{}' ({} bytes)", args.base, source_tex.len()),
    );

    // --- 2. the job description ---------------------------------------------
    let (jd_text, jd_origin) = resolve_jd(&ws, args)?;
    step(
        2,
        format!("Job description from {} ({} bytes)", jd_origin, jd_text.len()),
    );

    // --- 3. generate, compile and validate ----------------------------------
    step(
        3,
        format!(
            "Tailoring with {provider} ({model}), up to {} attempt(s)",
            1 + ai.max_retries
        ),
    );

    let request = TailorRequest {
        source_tex: source_tex.clone(),
        job_description: jd_text.clone(),
        company: args.company.clone(),
        role: args.role.clone(),
        required_terms: ai.required_terms.clone(),
        extra_instructions: ai.extra_instructions.clone(),
    };
    let document = Document::parse(&source_tex)?;
    let keywords = tailor::extract_keywords(&jd_text);
    let scratch = Scratch::new()?;
    let client = CliClient::new(&provider, &binary, &model, timeout);

    let attempts_allowed = (1 + ai.max_retries).max(1);
    let mut accepted: Option<Accepted> = None;
    let mut last_error = String::from("no attempt was made");

    for attempt in 1..=attempts_allowed {
        trace.attempts = attempt;

        let prompt = if attempt == 1 {
            tailor::build_prompt(&request)?
        } else {
            detail(&format!("retrying after: {last_error}"));
            tailor::build_repair_prompt(&request, &last_error)?
        };

        detail(&format!("attempt {attempt}/{attempts_allowed}: calling {provider}"));
        let raw = match client.complete(&prompt) {
            Ok(raw) => raw,
            Err(err) => {
                last_error = err.to_string();
                continue;
            }
        };

        let body = match tailor::extract_body(&raw) {
            Ok(body) => body,
            Err(err) => {
                last_error = err.to_string();
                continue;
            }
        };

        let tex = document.render(&body);
        if let Err(err) = tailor::lint(
            &tex,
            &document.body,
            &ai.required_terms,
            &config.document.required_sections,
        ) {
            last_error = err.to_string();
            continue;
        }

        let mut pdf = None;
        let mut docx = None;
        let mut pages = None;
        if !args.no_compile {
            detail(&format!(
                "compiling with {} (limit {} page(s))",
                config.compiler.engine, config.document.page_limit
            ));
            match compile_resume(&scratch, &tex, &config) {
                Ok(compiled) => {
                    trace.compiled = true;
                    pdf = Some(compiled.pdf);
                    docx = compiled.docx;
                    pages = Some(compiled.pages);
                }
                Err(err) => {
                    last_error = err.to_string();
                    continue;
                }
            }
            let count = pages.unwrap_or(1);
            if count > config.document.page_limit {
                last_error = format!(
                    "the compiled resume was {count} page(s) but the limit is {}; cut content, do not shrink the font",
                    config.document.page_limit
                );
                continue;
            }
            detail(&format!("compiled OK ({count} page(s))"));
        }

        accepted = Some(Accepted {
            tex,
            pdf,
            docx,
            pages,
        });
        break;
    }

    let accepted = match accepted {
        Some(accepted) => accepted,
        None => anyhow::bail!(
            "AI tailoring failed after {} attempt(s): {last_error}",
            trace.attempts
        ),
    };

    // --- 4. branch and commit -----------------------------------------------
    let branch_name = match &args.branch {
        Some(name) => name.clone(),
        None => tailor::slugify(
            args.company.as_deref(),
            args.role.as_deref(),
            jd_text.lines().next(),
        ),
    };

    // The branch is created only now: a failed tailoring run must not leave an
    // empty branch behind for the operator to clean up.
    if branch::exists(&ws, &branch_name) {
        if !args.force {
            anyhow::bail!(
                "Branch '{branch_name}' already exists. Pass --force to replace it."
            );
        }
        let auth = SystemAuthenticator::new();
        branch::delete(&ws, &branch_name, &auth)?;
    }

    step(
        4,
        format!("Creating branch '{}' from '{}'", branch_name, args.base),
    );
    branch::create_from(&ws, &branch_name, &args.base)?;
    rvm_tracker::job::save(
        &ws.branches_dir(),
        &branch_name,
        &JobMetadata {
            company: args.company.clone(),
            role: args.role.clone(),
            jd_text: Some(jd_text.clone()),
            notes: Some(format!(
                "Tailored by rvm {} ({} / {})",
                env!("CARGO_PKG_VERSION"),
                provider,
                model
            )),
            ..Default::default()
        },
    )?;

    // --- 5. commit, then optionally surface the result in the working tree ---
    step(
        5,
        format!("Committing to '{}'", branch_name),
    );
    let auth = SystemAuthenticator::new();
    let commit = commit::create_with_content(
        &ws,
        &branch_name,
        &commit_message(args, &jd_text),
        &accepted.tex,
        &auth,
    )?;

    let mut final_pdf = None;
    if let Some(source_pdf) = &accepted.pdf {
        // Keep a copy beside the branch snapshot so the PDF survives a later
        // checkout, then place it where the workspace expects its PDF.
        let branch_pdf = ws.branches_dir().join(&branch_name).join("snapshot.pdf");
        std::fs::copy(source_pdf, &branch_pdf)?;
        if args.no_checkout {
            final_pdf = Some(branch_pdf);
        } else {
            let dest = ws.find_pdf_path()?;
            std::fs::copy(source_pdf, &dest)?;
            final_pdf = Some(dest);
        }
    }

    // The DOCX follows the PDF exactly: same branch snapshot, same working-tree
    // file, same overwrite-in-place rule. A reader that only looks at one of the
    // two must never see a stale branch.
    let mut final_docx = None;
    if let Some(source_docx) = &accepted.docx {
        let branch_docx = ws.branches_dir().join(&branch_name).join("snapshot.docx");
        std::fs::copy(source_docx, &branch_docx)?;
        if args.no_checkout {
            final_docx = Some(branch_docx);
        } else {
            let dest = ws.find_docx_path()?;
            std::fs::copy(source_docx, &dest)?;
            final_docx = Some(dest);
        }
    }

    let checked_out = !args.no_checkout;
    if checked_out {
        // `checkout` writes the branch snapshot into the working .tex and moves
        // HEAD, so the operator is looking at the tailored resume immediately.
        branch::checkout(&ws, &branch_name)?;
    }

    Ok(TailorReport {
        branch: branch_name,
        base: args.base.clone(),
        commit: commit.hash.short().to_string(),
        pdf: final_pdf,
        docx: final_docx,
        pages: accepted.pages,
        keywords,
        attempts: trace.attempts,
        compiled: trace.compiled,
        checked_out,
    })
}

/// Read the committed snapshot of a branch.
///
/// Tailoring forks from what is *committed*, not from the working file: the
/// working file may belong to a different branch entirely, and silently baking
/// those edits into a job-specific resume would be indistinguishable from
/// corruption.
fn read_branch_tex(ws: &Workspace, base: &str) -> anyhow::Result<String> {
    let snapshot = ws.branches_dir().join(base).join("snapshot.tex");
    if !snapshot.exists() {
        anyhow::bail!(
            "Branch '{}' has no committed snapshot yet. Run `rvm commit -m \"...\"` on it first.",
            base
        );
    }
    Ok(std::fs::read_to_string(snapshot)?)
}

fn resolve_jd(ws: &Workspace, args: &TailorArgs) -> anyhow::Result<(String, String)> {
    if let Some(path) = &args.jd {
        let text = std::fs::read_to_string(path).map_err(|e| {
            anyhow::anyhow!("Cannot read job description '{}': {e}", path.display())
        })?;
        if text.trim().is_empty() {
            anyhow::bail!("Job description '{}' is empty.", path.display());
        }
        return Ok((text, path.display().to_string()));
    }

    if let Some(text) = &args.jd_text {
        if text.trim().is_empty() {
            anyhow::bail!("--jd-text is empty.");
        }
        return Ok((text.clone(), "--jd-text".to_string()));
    }

    let current = ws.current_branch()?;
    let meta = rvm_tracker::job::load(&ws.branches_dir(), &current)?;
    if let Some(jd) = meta.jd_text.filter(|text| !text.trim().is_empty()) {
        return Ok((jd, format!("job.json on branch '{current}'")));
    }

    anyhow::bail!(
        "No job description. Pass --jd <path> or --jd-text <text>, or store one in \
         .rvm/branches/<branch>/job.json."
    )
}

/// What one successful tailoring attempt produced, all inside the scratch dir.
struct Compiled {
    pdf: PathBuf,
    docx: Option<PathBuf>,
    pages: usize,
}

/// Compile a candidate resume in a scratch directory and report its page count.
///
/// The scratch directory matters: compiling in the workspace would overwrite the
/// operator's own `ManoharNalluri.pdf` with a draft that may still be rejected
/// by the page-limit check.
fn compile_resume(scratch: &Scratch, tex: &str, config: &RvmConfig) -> anyhow::Result<Compiled> {
    let tex_path = scratch.path().join("resume.tex");
    std::fs::write(&tex_path, tex)?;

    let compiler = rvm_compiler::Compiler::new(&config.compiler.engine)?;
    let result = compiler.compile(&tex_path)?;
    let pages = rvm_validator::page::page_count(&result.pdf_path)?;

    // Reported, never fatal: strict mode promotes whitespace warnings to
    // errors, and a tailorer that cannot satisfy them would fail every job.
    let report = rvm_validator::validate(&result.pdf_path, tex, config)?;
    for diagnostic in report.errors() {
        detail(&format!(
            "validation [{}]: {}",
            diagnostic.code, diagnostic.message
        ));
    }

    // Built here so a committed branch carries the same two artifacts the
    // operator's working tree gets. Never fatal: the PDF is what the page check
    // gates, and a machine without pandoc must still be able to tailor.
    let docx = if config.compiler.docx {
        match rvm_compiler::docx::convert(&tex_path) {
            Ok(converted) => Some(converted.docx_path),
            Err(e) => {
                detail(&format!("DOCX skipped: {e}"));
                None
            }
        }
    } else {
        None
    };

    Ok(Compiled {
        pdf: result.pdf_path,
        docx,
        pages,
    })
}

fn commit_message(args: &TailorArgs, jd_text: &str) -> String {
    let target = match (&args.company, &args.role) {
        (Some(company), Some(role)) => format!(" for {role} at {company}"),
        (Some(company), None) => format!(" for {company}"),
        (None, Some(role)) => format!(" for {role}"),
        // No metadata at all: the first line of the JD is usually the title.
        (None, None) => match jd_text.lines().find(|line| !line.trim().is_empty()) {
            Some(line) => format!(" for {}", line.trim()),
            None => String::new(),
        },
    };
    format!("Tailor resume{target} (via rvm {})", env!("CARGO_PKG_VERSION"))
}

fn step(index: u32, message: String) {
    eprintln!("[{index}/{STEPS}] {message}");
}

fn detail(message: &str) {
    eprintln!("      {message}");
}

/// A scratch directory that removes itself, so a run that fails mid-compile
/// cannot leave `rvm-tailor-*` directories behind in the temp dir.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> std::io::Result<Self> {
        let path = std::env::temp_dir().join(format!(
            "rvm-tailor-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_millis()
        ));
        std::fs::create_dir_all(&path)?;
        Ok(Self(path))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

// ---------------------------------------------------------------------------
// score
// ---------------------------------------------------------------------------

fn score() -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let ws = Workspace::discover(&cwd)?;

    println!("Scoring resume against JD...");
    match ws.find_tex_file() {
        Ok(tex_path) => {
            let tex = std::fs::read_to_string(&tex_path)?;
            let job = rvm_tracker::job::load(&ws.branches_dir(), &ws.current_branch()?)?;
            if let Some(jd) = &job.jd_text {
                let keywords = tailor::extract_keywords(jd);
                let result = rvm_ai::score::keyword_match(&tex, &keywords);
                println!("Keyword Match Score: {:.0}%", result.overall_score);
                println!("Matched: {}", result.matched_keywords.join(", "));
                if !result.missing_keywords.is_empty() {
                    println!("Missing: {}", result.missing_keywords.join(", "));
                }
            } else {
                println!("No job description found for current branch.");
                println!("Set one with the TUI or by editing .rvm/branches/<name>/job.json");
            }
        }
        Err(e) => {
            println!("{}", e);
        }
    }
    Ok(())
}
