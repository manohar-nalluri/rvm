mod commands;

use clap::Parser;
use tracing_subscriber::EnvFilter;

use commands::Cli;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();

    let cli = Cli::parse();
    if let Err(error) = commands::execute(cli) {
        // Print the message on its own rather than the `Debug` form of the
        // error chain: refusals such as a protected-branch rejection need to
        // read as instructions, not as a type dump.
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
