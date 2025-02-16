mod build;
mod cli;
mod init;
mod serve;
mod tag;
mod template;

use clap::Parser;
use scraps_libs::error::ScrapResult;

fn main() -> ScrapResult<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::SubCommands::Init { project_name } => cli::cmd::init::run(&project_name),
        cli::SubCommands::Build { verbose } => cli::cmd::build::run(verbose),
        cli::SubCommands::Serve => cli::cmd::serve::run(),
        cli::SubCommands::Tag => cli::cmd::tag::run(),
        cli::SubCommands::Template {
            template_command: template_commands,
        } => match template_commands {
            cli::TemplateSubCommands::Generate { template } => {
                cli::cmd::template::generate::run(template.name(), &template.title())
            }
        },
    }
}
