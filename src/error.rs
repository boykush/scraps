pub extern crate anyhow;

use std::path::PathBuf;

use thiserror::Error;

pub type ScrapsResult<T> = anyhow::Result<T>;

#[derive(Error, PartialEq, Debug)]
pub enum ScrapsError {
    #[error("Template error: {0}")]
    Template(#[from] TemplateError),

    #[error("Init error: {0}")]
    Init(#[from] InitError),

    #[error("Build error: {0}")]
    Build(#[from] BuildError),

    #[error("CLI error: {0}")]
    Cli(#[from] CliError),

    #[error("MCP error: {0}")]
    Mcp(#[from] McpError),

    #[error("Failed to read scrap: {0}")]
    ReadScrap(PathBuf),

    #[error("Failed to read scraps")]
    ReadScraps,
}

#[derive(Error, PartialEq, Debug)]
pub enum TemplateError {
    #[error("Failed to load template metadata")]
    LoadMetadata,

    #[error("Template title is required via command line or in template file")]
    RequiredTitle,

    #[error("Template not found: {0}")]
    NotFound(String),

    #[error("Failed to render template")]
    RenderFailure,

    #[error("Failed to write file")]
    WriteFailure,
}

#[derive(Error, PartialEq, Debug)]
pub enum InitError {
    #[error("Failed to initialize git repository")]
    GitInit,

    #[error("Failed to create directory")]
    CreateDirectory,

    #[error("Failed to write file: {0}")]
    WriteFailure(PathBuf),
}

#[derive(Error, PartialEq, Debug)]
pub enum BuildError {
    #[error("Failed to read README file")]
    ReadREADMEFile,

    #[error("Failed to get commited timestamp")]
    GitCommitedTs,

    #[error("Failed to create directory")]
    CreateDir,

    #[error("Failed to write file: {0}")]
    WriteFailure(PathBuf),

    #[error("Failed to render html")]
    RenderHtml,

    #[error("Failed to render css")]
    RenderCss,

    #[error("Failed to render json")]
    RenderJson,
}

#[derive(Error, PartialEq, Debug)]
pub enum ServeError {
    #[error("Failed to load file")]
    LoadFile,
}

#[derive(Error, PartialEq, Debug)]
pub enum CliError {
    #[error("Not display data on cli")]
    Display,

    #[error("Failed when load config")]
    ConfigLoad,

    #[error(
        "Invalid .scraps.toml format. See: https://boykush.github.io/scraps/Reference/Configuration"
    )]
    InvalidConfigFormat,

    #[error("Missing [ssg] section in .scraps.toml. This section is required for build and serve commands. See: https://boykush.github.io/scraps/Reference/Configuration")]
    MissingSsgSection,
}

#[derive(Error, PartialEq, Debug)]
pub enum McpError {
    #[error("Failed to setup tracing for MCP server: {0}")]
    TracingSetup(String),

    #[error("MCP service error: {0}")]
    ServiceError(String),

    #[error("Failed to create tokio runtime: {0}")]
    RuntimeCreation(String),
}
