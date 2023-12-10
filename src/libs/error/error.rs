use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ScrapError {
    #[error("Not load config")]
    ConfigLoadError,

    #[error("Not load file")]
    FileLoadError,

    #[error("Failed write file")]
    FileWriteError,

    #[error("Failed when render to html")]
    PublicRenderError,

    #[error("Failed git init. git is required")]
    GitInitError,

    #[error("Failed git log. git is required")]
    GitLogError,

    #[error("Not found")]
    NotFoundError,
}
