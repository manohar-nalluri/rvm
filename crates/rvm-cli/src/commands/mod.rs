pub mod ai;
pub mod archive;
pub mod branch;
pub mod checkout;
pub mod commit;
pub mod diff;
pub mod export;
pub mod init;
pub mod log;
pub mod merge;
pub mod status;
pub mod tui;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "rvm",
    about = "RVM - Resume Version Manager",
    long_about = "A git-like version control system for managing resume versions, \
                  with LaTeX compilation, AI-powered tailoring, and job tracking.",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new RVM workspace in the current directory
    Init,

    /// Create, list, or manage branches
    Branch {
        /// Name of the new branch to create
        name: Option<String>,

        /// List all active branches
        #[arg(short, long)]
        list: bool,

        /// List archived branches
        #[arg(long)]
        archived: bool,
    },

    /// Switch to a branch; updates the .tex file and recompiles PDF
    Checkout {
        /// Branch name to switch to
        name: String,
    },

    /// Snapshot the current .tex file with a commit message
    Commit {
        /// Commit message
        #[arg(short, long)]
        message: String,
    },

    /// Show differences between two branches or commits
    Diff {
        /// First branch/commit (defaults to current)
        a: Option<String>,
        /// Second branch/commit
        b: Option<String>,
    },

    /// Merge a branch into the current branch with conflict resolution
    Merge {
        /// Branch to merge from
        branch: String,
    },

    /// View commit history for the current branch
    Log,

    /// Show job tracking dashboard across all active branches
    Status,

    /// Move a branch to archived state
    Archive {
        /// Branch to archive
        branch: String,
    },

    /// Bundle resume PDF, cover letter, and references into a zip
    Export {
        /// Branch to export (defaults to current)
        branch: Option<String>,
    },

    /// AI-powered operations
    #[command(subcommand)]
    Ai(ai::AiCommands),

    /// Launch the interactive terminal UI
    Tui,
}

pub fn execute(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Init => init::execute(),
        Commands::Branch {
            name,
            list,
            archived,
        } => branch::execute(name.as_deref(), list, archived),
        Commands::Checkout { name } => checkout::execute(&name),
        Commands::Commit { message } => commit::execute(&message),
        Commands::Diff { a, b } => diff::execute(a.as_deref(), b.as_deref()),
        Commands::Merge { branch } => merge::execute(&branch),
        Commands::Log => log::execute(),
        Commands::Status => status::execute(),
        Commands::Archive { branch } => archive::execute(&branch),
        Commands::Export { branch } => export::execute(branch.as_deref()),
        Commands::Ai(cmd) => ai::execute(cmd),
        Commands::Tui => tui::execute(),
    }
}
