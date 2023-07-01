use std::{path::PathBuf, process::Command};

use anyhow::Context;

use super::error::{error::ScrapError, result::ScrapResult};

pub trait GitCommand {
    fn init(&self, path: &PathBuf) -> ScrapResult<()>;
}

pub struct GitCommandImpl {}

impl GitCommandImpl {
    pub fn new() -> GitCommandImpl {
        GitCommandImpl {}
    }
}

impl GitCommand for GitCommandImpl {
    fn init(&self, path: &PathBuf) -> ScrapResult<()> {
        Command::new("git")
            .arg("init")
            .current_dir(path)
            .output()
            .map(|_| ())
            .context(ScrapError::GitInitError)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub struct GitCommandTest {}

    impl GitCommandTest {
        pub fn _new() -> GitCommandTest {
            GitCommandTest {}
        }
    }

    impl GitCommand for GitCommandTest {
        fn init(&self, _path: &PathBuf) -> ScrapResult<()> {
            Ok(())
        }
    }
}
