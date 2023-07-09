use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use anyhow::Context;

use super::error::{error::ScrapError, result::ScrapResult};

pub trait GitCommand {
    fn init(&self, path: &PathBuf) -> ScrapResult<()>;
    fn commited_ts(&self, path: &PathBuf) -> ScrapResult<Option<i64>>;
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
            .current_dir(path)
            .arg("init")
            .output()
            .map(|_| ())
            .context(ScrapError::GitInitError)
    }

    fn commited_ts(&self, path: &PathBuf) -> ScrapResult<Option<i64>> {
        let output = Command::new("git")
            .arg("log")
            .arg("-1")
            .arg("--format=%ct")
            .arg(path)
            .stdout(Stdio::piped())
            .output()
            .context(ScrapError::GitLogError)?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let commited_ts = output_str
            .trim()
            .parse::<i64>()
            .map_or_else(|_| None, |s| Some(s));
        Ok(commited_ts)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub struct GitCommandTest {}

    impl GitCommandTest {
        pub fn new() -> GitCommandTest {
            GitCommandTest {}
        }
    }

    impl GitCommand for GitCommandTest {
        fn init(&self, _path: &PathBuf) -> ScrapResult<()> {
            Ok(())
        }
        fn commited_ts(&self, _path: &PathBuf) -> ScrapResult<Option<i64>> {
            Ok(Some(0))
        }
    }
}
