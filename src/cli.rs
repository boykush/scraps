use clap::{Parser, Subcommand, ValueEnum};
use clap_verbosity_flag::{Verbosity, WarnLevel};
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

        #[arg(
            long,
            help = "Include git-derived metadata (commited_ts) in HTML output and template variables"
        )]
        git: bool,
    },

    #[command(about = "Get the markdown body of a scrap by title")]
    Get {
        title: String,

        #[arg(long, help = "Disambiguate title across contexts")]
        ctx: Option<String>,

        #[arg(long, help = "Output as JSON")]
        json: bool,
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
    Serve {
        #[arg(
            long,
            help = "Include git-derived metadata (commited_ts) in HTML output and template variables"
        )]
        git: bool,
    },

    #[command(about = "Tag commands")]
    Tag {
        #[command(subcommand)]
        tag_command: TagSubCommands,
    },

    #[command(about = "MCP server commands")]
    Mcp {
        #[command(subcommand)]
        mcp_command: McpSubCommands,
    },
}

#[derive(Subcommand)]
pub enum TagSubCommands {
    #[command(about = "List tags")]
    List {
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },

    #[command(about = "List scraps that reference the specified tag")]
    Backlinks {
        tag: String,

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
