mod build;
mod cli;
mod libs;

use clap::Parser;
use libs::error::result::ScrapResult;

fn main() -> ScrapResult<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::SubCommands::Build => cli::cmd::build::run(),
    }
}
