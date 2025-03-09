pub extern crate anyhow;

use std::path::PathBuf;

use thiserror::Error;

pub type ScrapsResult<T> = anyhow::Result<T>;

#[derive(Error, Debug)]
pub enum ScrapsError {
    #[error("Template error: {0}")]
    Template(#[from] TemplateError),

    #[error("Init error: {0}")]
    Init(#[from] InitError),

    #[error("Build error: {0}")]
    Build(#[from] BuildError),

    #[error("Failed to read scraps")]
    ReadScraps,

    #[error("Failed when load config")]
    ConfigLoad,

    #[error("Not load file")]
    FileLoad,

    #[error("Failed write file")]
    FileWrite,

    #[error("Not display data on cli")]
    CliDisplay,
}

#[derive(Error, Debug)]
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

#[derive(Error, Debug)]
pub enum InitError {
    #[error("Failed to initialize git repository")]
    GitInit,

    #[error("Failed to create directory or file")]
    CreateDirectoryOrFile,
}

#[derive(Error, Debug)]
pub enum BuildError {
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
