mod build;
mod cli;
mod init;
mod libs;
mod serve;

use clap::Parser;
use libs::error::ScrapResult;

fn main() -> ScrapResult<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::SubCommands::Init { project_name } => cli::cmd::init::run(&project_name),
        cli::SubCommands::Build => cli::cmd::build::run(),
        cli::SubCommands::Serve => cli::cmd::build::run().and(cli::cmd::serve::run()),
    }
}
