mod cli;
mod constants;
mod error;
mod input;
mod mcp;
mod output;
mod service;
mod usecase;

#[cfg(test)]
mod test_fixtures;

use clap::Parser;
use error::McpError;

fn main() -> error::ScrapsResult<()> {
    let cli::Cli {
        directory,
        path,
        command,
    } = cli::Cli::parse();
    let directory = cli::Cli::resolve_directory(directory.as_deref(), path.as_deref());

    match command {
        cli::SubCommands::Init => cli::cmd::init::run(directory),
        cli::SubCommands::Build { verbose, git } => cli::cmd::build::run(verbose, git, directory),
        cli::SubCommands::Get { title, ctx, json } => cli::cmd::get::run(
            &title,
            ctx.as_deref(),
            json,
            directory,
            &mut std::io::stdout(),
        ),
        cli::SubCommands::Lint { rules } => {
            let rule_names: Vec<_> = rules.into_iter().map(Into::into).collect();
            cli::cmd::lint::run(directory, &rule_names)
        }
        cli::SubCommands::Links { title, ctx, json } => cli::cmd::links::run(
            &title,
            ctx.as_deref(),
            json,
            directory,
            &mut std::io::stdout(),
        ),
        cli::SubCommands::Backlinks { title, ctx, json } => cli::cmd::backlinks::run(
            &title,
            ctx.as_deref(),
            json,
            directory,
            &mut std::io::stdout(),
        ),
        cli::SubCommands::Search {
            query,
            num,
            logic,
            json,
        } => cli::cmd::search::run(
            &query,
            num,
            logic.into(),
            json,
            directory,
            &mut std::io::stdout(),
        ),
        cli::SubCommands::Serve { git } => cli::cmd::serve::run(git, directory),
        cli::SubCommands::Tag { tag_command } => match tag_command {
            cli::TagSubCommands::List { json } => {
                cli::cmd::tag::list::run(json, directory, &mut std::io::stdout())
            }
            cli::TagSubCommands::Backlinks { tag, json } => {
                cli::cmd::tag::backlinks::run(&tag, json, directory, &mut std::io::stdout())
            }
        },
        cli::SubCommands::Todo { status, json } => {
            cli::cmd::todo::run(status.into(), json, directory, &mut std::io::stdout())
        }
        cli::SubCommands::Mcp { mcp_command } => match mcp_command {
            cli::McpSubCommands::Serve => {
                let runtime = tokio::runtime::Runtime::new()
                    .map_err(|e| McpError::RuntimeCreation(e.to_string()))?;
                runtime.block_on(cli::cmd::mcp::serve::run(directory))
            }
        },
    }
}
