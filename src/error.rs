pub extern crate anyhow;

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ScrapsError {
    #[error("Template error: {0}")]
    Template(#[from] TemplateError),

    #[error("Failed when load config")]
    ConfigLoad,

    #[error("Not load file")]
    FileLoad,

    #[error("Failed write file")]
    FileWrite,

    #[error("Not display data on cli")]
    CliDisplay,

    #[error("Failed git init. git is required")]
    GitInit,

    #[error("Failed git log. git is required")]
    GitLog,

    #[error("Failed when convert from str")]
    FromStrErr,
}

#[derive(Error, Debug, PartialEq)]
pub enum TemplateError {
    #[error("Failed to load template metadata")]
    MetadataLoad,

    #[error("Template title is required via command line or in template file")]
    RequiredTitle,

    #[error("Template not found: {0}")]
    NotFound(String),

    #[error("Failed to render template: {0}")]
    RenderFailure(String),
}

pub type ScrapsResult<T> = anyhow::Result<T>;
