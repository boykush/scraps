use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ScrapError {
    #[error("Not load config")]
    ConfigLoadError,

    #[error("Not load file")]
    FileLoadError,

    #[error("Failed when render to html")]
    PublicRenderError,

    #[error("Failed write file")]
    FileWriteError,

    #[error("Failed git init. git is required")]
    GitInitError,
}
