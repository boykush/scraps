mod build;
mod cli;
mod init;
mod libs;
mod serve;
mod tag;

use clap::Parser;
use scraps_libs::error::ScrapResult;

fn main() -> ScrapResult<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::SubCommands::Init { project_name } => cli::cmd::init::run(&project_name),
        cli::SubCommands::Build => cli::cmd::build::run(),
        cli::SubCommands::Serve => cli::cmd::serve::run(),
        cli::SubCommands::Tag => cli::cmd::tag::run(),
    }
}
