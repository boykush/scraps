pub extern crate anyhow;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScrapsError {
    #[error("Template error: {0}")]
    Template(#[from] TemplateError),

    #[error("Init error: {0}")]
    Init(#[from] InitError),

    #[error("Failed when load config")]
    ConfigLoad,

    #[error("Not load file")]
    FileLoad,

    #[error("Failed write file")]
    FileWrite,

    #[error("Not display data on cli")]
    CliDisplay,

    #[error("Failed when convert from str")]
    FromStrErr,
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

pub type ScrapsResult<T> = anyhow::Result<T>;
