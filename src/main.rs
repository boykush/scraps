mod cli;
mod constants;
mod error;
mod mcp;
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
        cli::SubCommands::Serve => cli::cmd::serve::run(cli.path.as_deref()),
        cli::SubCommands::Tag => cli::cmd::tag::run(cli.path.as_deref()),
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
