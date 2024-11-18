use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ScrapError {
    #[error("Not load config")]
    ConfigLoad,

    #[error("Not load file")]
    FileLoad,

    #[error("Failed write file")]
    FileWrite,

    #[error("Failed when render to html")]
    PublicRender,

    #[error("Not display data on cli")]
    CliDisplay,

    #[error("Failed git init. git is required")]
    GitInit,

    #[error("Failed git log. git is required")]
    GitLog,
}

pub type ScrapResult<T> = anyhow::Result<T>;
