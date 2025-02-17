pub extern crate anyhow;

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ScrapError {
    #[error("Failed when load config")]
    ConfigLoad,

    #[error("Not load file")]
    FileLoad,

    #[error("Failed write file")]
    FileWrite,

    #[error("Failed when render to html")]
    PublicRender,

    #[error("Failed when load template metadata")]
    TemplateMetadataLoad,

    #[error("Template title are required to be entered via the command line or defined in the template file.")]
    RequiredTemplateTitle,

    #[error("Not found template for generate")]
    NotFoundTemplate,

    #[error("Not display data on cli")]
    CliDisplay,

    #[error("Failed git init. git is required")]
    GitInit,

    #[error("Failed git log. git is required")]
    GitLog,
}

pub type ScrapResult<T> = anyhow::Result<T>;
