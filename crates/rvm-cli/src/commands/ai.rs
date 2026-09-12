use clap::Subcommand;

use rvm_core::Workspace;

#[derive(Subcommand)]
pub enum AiCommands {
    /// Tailor resume to the associated job description
    Tailor,

    /// Create a new branch and generate a tailored resume from a JD file
    Create {
        /// Path to the job description file
        #[arg(long)]
        jd: String,
    },

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

pub fn execute(cmd: AiCommands) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let ws = Workspace::discover(&cwd)?;

    match cmd {
        AiCommands::Tailor => {
            println!("AI tailoring resume on branch '{}'...", ws.current_branch()?);
            println!("(AI integration requires Claude Code runtime)");
        }
        AiCommands::Create { jd } => {
            println!("Creating new branch from JD: {}", jd);
            println!("(AI integration requires Claude Code runtime)");
        }
        AiCommands::Score => {
            println!("Scoring resume against JD...");
            // Perform a local keyword match as a fallback
            match ws.find_tex_file() {
                Ok(tex_path) => {
                    let tex = std::fs::read_to_string(&tex_path)?;
                    let job = rvm_tracker::job::load(&ws.branches_dir(), &ws.current_branch()?)?;
                    if let Some(jd) = &job.jd_text {
                        let keywords = rvm_ai::tailor::extract_keywords(jd);
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
        }
        AiCommands::CoverLetter => {
            println!("Generating cover letter...");
            println!("(AI integration requires Claude Code runtime)");
        }
        AiCommands::Prep => {
            println!("Generating interview prep...");
            println!("(AI integration requires Claude Code runtime)");
        }
        AiCommands::Review => {
            println!("Reviewing resume...");
            println!("(AI integration requires Claude Code runtime)");
        }
        AiCommands::Init => {
            let skills_dir = ws.skills_dir();
            let manager = rvm_ai::skill::SkillManager::new(&skills_dir);
            manager.init_default_skills()?;
            println!("Initialized AI skills in {}", skills_dir.display());
        }
    }

    Ok(())
}
