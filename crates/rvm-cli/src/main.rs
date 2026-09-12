mod commands;

use clap::Parser;
use tracing_subscriber::EnvFilter;

use commands::Cli;

fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();

    let cli = Cli::parse();
    commands::execute(cli)
}
