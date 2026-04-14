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
        cli::SubCommands::Build { verbose } => cli::cmd::build::run(verbose, cli.path.as_deref()),
        cli::SubCommands::Lint { rules } => {
            let rule_names: Vec<_> = rules.into_iter().map(Into::into).collect();
            cli::cmd::lint::run(cli.path.as_deref(), &rule_names)
        }
        cli::SubCommands::Serve => cli::cmd::serve::run(cli.path.as_deref()),
        cli::SubCommands::Tag { tag_command } => match tag_command {
            cli::TagSubCommands::List { json } => {
                cli::cmd::tag::list::run(json, cli.path.as_deref(), &mut std::io::stdout())
            }
        },
        cli::SubCommands::Template {
            template_command: template_commands,
        } => match template_commands {
            cli::TemplateSubCommands::Generate { template } => cli::cmd::template::generate::run(
                template.name(),
                &template.title(),
                cli.path.as_deref(),
            ),
            cli::TemplateSubCommands::List => cli::cmd::template::list::run(cli.path.as_deref()),
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
