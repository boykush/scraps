use clap::{Args, Parser, Subcommand};
use clap_verbosity_flag::{Verbosity, WarnLevel};
use scraps_libs::model::title::Title;
use std::path::PathBuf;

pub mod cmd;
mod config;
mod display;
pub mod path_resolver;
mod progress;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(
        short = 'p',
        long = "path",
        global = true,
        env = "SCRAPS_PROJECT_PATH",
        help = "Specify the project directory path"
    )]
    pub path: Option<PathBuf>,

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

    #[command(about = "Lint scraps for broken links")]
    Lint,

    #[command(about = "Serve the site with build scraps")]
    Serve,

    #[command(about = "List a tags")]
    Tag,

    #[command(about = "Template of scraps")]
    Template {
        #[command(subcommand)]
        template_command: TemplateSubCommands,
    },

    #[command(about = "MCP server commands")]
    Mcp {
        #[command(subcommand)]
        mcp_command: McpSubCommands,
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

#[derive(Subcommand)]
pub enum McpSubCommands {
    #[command(about = "Start MCP server with stdio transport")]
    Serve,
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
