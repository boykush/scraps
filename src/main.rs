mod build;
mod cli;
mod init;
mod serve;
mod tag;

use clap::Parser;
use scraps_libs::error::ScrapResult;

fn main() -> ScrapResult<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::SubCommands::Init { project_name } => cli::cmd::init::run(&project_name),
        cli::SubCommands::Build { verbose } => cli::cmd::build::run(verbose),
        cli::SubCommands::Serve => cli::cmd::serve::run(),
        cli::SubCommands::Tag => cli::cmd::tag::run(),
    }
}
