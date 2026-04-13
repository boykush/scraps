use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_verbosity_flag::{Verbosity, WarnLevel};
use scraps_libs::model::title::Title;
use std::path::PathBuf;

use crate::usecase::lint::rule::LintRuleName;

pub mod cmd;
mod config;
mod display;
mod json;
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

    #[command(about = "Lint scraps for wiki-link quality issues")]
    Lint {
        #[arg(
            short = 'r',
            long = "rule",
            help = "Run only the specified lint rule(s)"
        )]
        rules: Vec<CliLintRuleName>,
    },

    #[command(about = "Serve the site with build scraps")]
    Serve,

    #[command(about = "Tag commands")]
    Tag {
        #[command(subcommand)]
        tag_command: TagSubCommands,
    },

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
pub enum TagSubCommands {
    #[command(about = "List tags")]
    List {
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },
}

#[derive(Subcommand)]
pub enum McpSubCommands {
    #[command(about = "Start MCP server with stdio transport")]
    Serve,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum CliLintRuleName {
    #[value(name = "dead-end")]
    DeadEnd,
    #[value(name = "lonely")]
    Lonely,
    #[value(name = "self-link")]
    SelfLink,
    #[value(name = "overlinking")]
    Overlinking,
    #[value(name = "singleton-tag")]
    SingletonTag,
}

impl From<CliLintRuleName> for LintRuleName {
    fn from(cli: CliLintRuleName) -> Self {
        match cli {
            CliLintRuleName::DeadEnd => LintRuleName::DeadEnd,
            CliLintRuleName::Lonely => LintRuleName::Lonely,
            CliLintRuleName::SelfLink => LintRuleName::SelfLink,
            CliLintRuleName::Overlinking => LintRuleName::Overlinking,
            CliLintRuleName::SingletonTag => LintRuleName::SingletonTag,
        }
    }
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
        self.scrap_title.as_ref().map(|s| s.as_str().into())
    }
}
