use clap::{Args, Parser, Subcommand};
use clap_verbosity_flag::{Verbosity, WarnLevel};
use scraps_libs::model::title::Title;

pub mod cmd;
mod config;
mod display;
mod progress;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: SubCommands,
}

#[derive(Subcommand)]
pub enum SubCommands {
    #[command(about = "Build scraps")]
    Build {
        #[command(flatten)]
        verbose: Verbosity<WarnLevel>,
    },

    #[command(about = "Init scraps project")]
    Init { project_name: String },

    #[command(about = "Serve the site with build scraps")]
    Serve,

    #[command(about = "List a tags")]
    Tag,

    #[command(about = "Template of scraps")]
    Template {
        #[command(subcommand)]
        template_command: TemplateSubCommands,
    },
}

#[derive(Subcommand)]
pub enum TemplateSubCommands {
    #[command(about = "Generate scrap from template")]
    Generate {
        #[command(flatten)]
        template: Template,
    },

    #[command(about = "List templates")]
    List,
}

#[derive(Args, Clone)]
pub struct Template {
    template_name: String,

    #[arg(short = 't', long, help = "This overrides the template metadata.")]
    scrap_title: Option<String>,
}

impl Template {
    pub fn name(&self) -> &str {
        &self.template_name
    }

    pub fn title(&self) -> Option<Title> {
        self.scrap_title.clone().map(|s| s.as_str().into())
    }
}
