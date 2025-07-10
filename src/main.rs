mod cli;
mod error;
mod service;
mod usecase;

use clap::Parser;

fn main() -> error::ScrapsResult<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::SubCommands::Init { project_name } => cli::cmd::init::run(&project_name),
        cli::SubCommands::Build { verbose } => cli::cmd::build::run(verbose),
        cli::SubCommands::Serve => cli::cmd::serve::run(),
        cli::SubCommands::Search { query, num } => {
            cli::cmd::search::run(&query, num.unwrap_or(100))
        }
        cli::SubCommands::Tag => cli::cmd::tag::run(),
        cli::SubCommands::Template {
            template_command: template_commands,
        } => match template_commands {
            cli::TemplateSubCommands::Generate { template } => {
                cli::cmd::template::generate::run(template.name(), &template.title())
            }
            cli::TemplateSubCommands::List => cli::cmd::template::list::run(),
        },
    }
}
