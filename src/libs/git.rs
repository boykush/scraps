use std::{
    path::Path,
    process::{Command, Stdio},
};

use scraps_libs::error::{anyhow::Context, ScrapError, ScrapResult};

pub trait GitCommand {
    fn init(&self, path: &Path) -> ScrapResult<()>;
    fn commited_ts(&self, path: &Path) -> ScrapResult<Option<i64>>;
}

pub struct GitCommandImpl {}

impl GitCommandImpl {
    pub fn new() -> GitCommandImpl {
        GitCommandImpl {}
    }
}

impl GitCommand for GitCommandImpl {
    fn init(&self, path: &Path) -> ScrapResult<()> {
        Command::new("git")
            .current_dir(path)
            .arg("init")
            .output()
            .map(|_| ())
            .context(ScrapError::GitInit)
    }

    fn commited_ts(&self, path: &Path) -> ScrapResult<Option<i64>> {
        let output = Command::new("git")
            .arg("log")
            .arg("-1")
            .arg("--format=%ct")
            .arg(path)
            .stdout(Stdio::piped())
            .output()
            .context(ScrapError::GitLog)?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let commited_ts = output_str.trim().parse::<i64>().ok();
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
        fn init(&self, _path: &Path) -> ScrapResult<()> {
            Ok(())
        }
        fn commited_ts(&self, _path: &Path) -> ScrapResult<Option<i64>> {
            Ok(Some(0))
        }
    }
}
