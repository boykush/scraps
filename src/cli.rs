use clap::{Parser, Subcommand, ValueEnum};
use clap_verbosity_flag::{Verbosity, WarnLevel};
use std::path::PathBuf;

use crate::usecase::lint::rule::LintRuleName;
use crate::usecase::todo::usecase::StatusFilter;
use scraps_libs::search::engine::SearchLogic;

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
        short = 'C',
        long = "directory",
        global = true,
        env = "SCRAPS_DIRECTORY",
        value_name = "DIR",
        help = "Run as if started in <DIR> (instead of the current directory)"
    )]
    pub directory: Option<PathBuf>,

    #[arg(
        short = 'p',
        long = "path",
        global = true,
        env = "SCRAPS_PROJECT_PATH",
        value_name = "DIR",
        hide = true,
        help = "[DEPRECATED] Use -C/--directory instead. Will be removed in v1.1."
    )]
    pub path: Option<PathBuf>,

    #[command(subcommand)]
    pub command: SubCommands,
}

impl Cli {
    /// Resolve the effective working directory, preferring `-C/--directory`.
    ///
    /// Emits a deprecation warning to stderr when `-p/--path` (or
    /// `SCRAPS_PROJECT_PATH`) is used.
    pub fn resolve_directory<'a>(
        directory: Option<&'a std::path::Path>,
        path: Option<&'a std::path::Path>,
    ) -> Option<&'a std::path::Path> {
        if path.is_some() {
            eprintln!(
                "warning: -p/--path (and SCRAPS_PROJECT_PATH) is deprecated and will be removed in v1.1. Use -C/--directory (or SCRAPS_DIRECTORY) instead."
            );
        }
        directory.or(path)
    }
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

        #[arg(
            long,
            value_name = "HEADING",
            help = "Restrict the output to the section under the given heading text"
        )]
        heading: Option<String>,

        #[arg(
            long,
            value_name = "FIELDS",
            num_args = 0..=1,
            default_missing_value = "",
            help = "Output as JSON. Optionally accepts a comma-separated field projection (title,ctx,body,headings,code_blocks)."
        )]
        json: Option<String>,
    },

    #[command(about = "Write .scraps.toml to the project directory")]
    Init,

    #[command(about = "Lint scraps for wiki-link quality issues")]
    Lint {
        #[arg(
            short = 'r',
            long = "rule",
            help = "Run only the specified lint rule(s)"
        )]
        rules: Vec<CliLintRuleName>,
    },

    #[command(about = "List outbound wiki-links from a scrap")]
    Links {
        title: String,

        #[arg(long, help = "Disambiguate title across contexts")]
        ctx: Option<String>,

        #[arg(long, help = "Output as JSON")]
        json: bool,
    },

    #[command(about = "List inbound wiki-links (backlinks) to a scrap")]
    Backlinks {
        title: String,

        #[arg(long, help = "Disambiguate title across contexts")]
        ctx: Option<String>,

        #[arg(long, help = "Output as JSON")]
        json: bool,
    },

    #[command(about = "Search scraps by query (fuzzy)")]
    Search {
        query: String,

        #[arg(long, default_value_t = 100, help = "Maximum number of results")]
        num: usize,

        #[arg(
            long,
            value_enum,
            default_value_t = CliSearchLogic::Or,
            help = "Search logic for multi-keyword queries"
        )]
        logic: CliSearchLogic,

        #[arg(long, help = "Output as JSON")]
        json: bool,
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

    #[command(about = "Aggregate markdown task list items across the wiki")]
    Todo {
        #[arg(
            long,
            value_enum,
            default_value_t = CliTodoStatus::Open,
            help = "Filter task items by status"
        )]
        status: CliTodoStatus,

        #[arg(long, help = "Output as JSON")]
        json: bool,
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
    #[value(name = "broken-link")]
    BrokenLink,
    #[value(name = "broken-heading-ref")]
    BrokenHeadingRef,
    #[value(name = "stale-by-git")]
    StaleByGit,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CliSearchLogic {
    #[value(name = "and")]
    And,
    #[value(name = "or")]
    Or,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CliTodoStatus {
    #[value(name = "open")]
    Open,
    #[value(name = "done")]
    Done,
    #[value(name = "deferred")]
    Deferred,
    #[value(name = "all")]
    All,
}

impl From<CliTodoStatus> for StatusFilter {
    fn from(cli: CliTodoStatus) -> Self {
        match cli {
            CliTodoStatus::Open => StatusFilter::Open,
            CliTodoStatus::Done => StatusFilter::Done,
            CliTodoStatus::Deferred => StatusFilter::Deferred,
            CliTodoStatus::All => StatusFilter::All,
        }
    }
}

impl From<CliSearchLogic> for SearchLogic {
    fn from(cli: CliSearchLogic) -> Self {
        match cli {
            CliSearchLogic::And => SearchLogic::And,
            CliSearchLogic::Or => SearchLogic::Or,
        }
    }
}

impl From<CliLintRuleName> for LintRuleName {
    fn from(cli: CliLintRuleName) -> Self {
        match cli {
            CliLintRuleName::DeadEnd => LintRuleName::DeadEnd,
            CliLintRuleName::Lonely => LintRuleName::Lonely,
            CliLintRuleName::SelfLink => LintRuleName::SelfLink,
            CliLintRuleName::Overlinking => LintRuleName::Overlinking,
            CliLintRuleName::BrokenLink => LintRuleName::BrokenLink,
            CliLintRuleName::BrokenHeadingRef => LintRuleName::BrokenHeadingRef,
            CliLintRuleName::StaleByGit => LintRuleName::StaleByGit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn resolve_directory_prefers_directory_over_path() {
        let directory = PathBuf::from("/dir");
        let path = PathBuf::from("/path");
        let resolved = Cli::resolve_directory(Some(&directory), Some(&path));
        assert_eq!(resolved, Some(Path::new("/dir")));
    }

    #[test]
    fn resolve_directory_falls_back_to_path_when_directory_absent() {
        let path = PathBuf::from("/path");
        let resolved = Cli::resolve_directory(None, Some(&path));
        assert_eq!(resolved, Some(Path::new("/path")));
    }

    #[test]
    fn resolve_directory_uses_directory_when_path_absent() {
        let directory = PathBuf::from("/dir");
        let resolved = Cli::resolve_directory(Some(&directory), None);
        assert_eq!(resolved, Some(Path::new("/dir")));
    }

    #[test]
    fn resolve_directory_returns_none_when_both_absent() {
        let resolved = Cli::resolve_directory(None, None);
        assert_eq!(resolved, None);
    }
}
