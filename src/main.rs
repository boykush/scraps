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
    let cli = cli::Cli::parse();

    match cli.command {
        cli::SubCommands::Init { project_name } => {
            cli::cmd::init::run(&project_name, cli.path.as_deref())
        }
        cli::SubCommands::Build { verbose, git } => {
            cli::cmd::build::run(verbose, git, cli.path.as_deref())
        }
        cli::SubCommands::Get { title, ctx, json } => cli::cmd::get::run(
            &title,
            ctx.as_deref(),
            json,
            cli.path.as_deref(),
            &mut std::io::stdout(),
        ),
        cli::SubCommands::Lint { rules } => {
            let rule_names: Vec<_> = rules.into_iter().map(Into::into).collect();
            cli::cmd::lint::run(cli.path.as_deref(), &rule_names)
        }
        cli::SubCommands::Links { title, ctx, json } => cli::cmd::links::run(
            &title,
            ctx.as_deref(),
            json,
            cli.path.as_deref(),
            &mut std::io::stdout(),
        ),
        cli::SubCommands::Serve { git } => cli::cmd::serve::run(git, cli.path.as_deref()),
        cli::SubCommands::Tag { tag_command } => match tag_command {
            cli::TagSubCommands::List { json } => {
                cli::cmd::tag::list::run(json, cli.path.as_deref(), &mut std::io::stdout())
            }
            cli::TagSubCommands::Backlinks { tag, json } => cli::cmd::tag::backlinks::run(
                &tag,
                json,
                cli.path.as_deref(),
                &mut std::io::stdout(),
            ),
        },
        cli::SubCommands::Mcp { mcp_command } => match mcp_command {
            cli::McpSubCommands::Serve => {
                let runtime = tokio::runtime::Runtime::new()
                    .map_err(|e| McpError::RuntimeCreation(e.to_string()))?;
                runtime.block_on(cli::cmd::mcp::serve::run(cli.path.as_deref()))
            }
        },
    }
}
