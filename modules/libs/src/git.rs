use std::{
    io,
    path::Path,
    process::{Command, Stdio},
};

pub trait GitCommand {
    fn init(&self, path: &Path) -> io::Result<()>;
    fn commited_ts(&self, path: &Path) -> io::Result<Option<i64>>;
    /// Whether `path` lives inside a git working tree.
    ///
    /// Returns `Ok(false)` when git reports the path is not inside a working
    /// tree. A missing `git` binary is also reported as `Ok(false)` so that
    /// callers can degrade gracefully without distinguishing the two cases.
    fn is_git_repository(&self, path: &Path) -> io::Result<bool>;
}

#[derive(Clone, Copy)]
pub struct GitCommandImpl {}

impl GitCommandImpl {
    pub fn new() -> GitCommandImpl {
        GitCommandImpl {}
    }
}

impl Default for GitCommandImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl GitCommand for GitCommandImpl {
    fn init(&self, path: &Path) -> io::Result<()> {
        Command::new("git")
            .current_dir(path)
            .arg("init")
            .output()
            .map(|_| ())
    }

    fn commited_ts(&self, path: &Path) -> io::Result<Option<i64>> {
        let output = Command::new("git")
            .arg("log")
            .arg("-1")
            .arg("--format=%ct")
            .arg(path)
            .stdout(Stdio::piped())
            .output()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let commited_ts = output_str.trim().parse::<i64>().ok();
        Ok(commited_ts)
    }

    fn is_git_repository(&self, path: &Path) -> io::Result<bool> {
        let result = Command::new("git")
            .current_dir(path)
            .arg("rev-parse")
            .arg("--is-inside-work-tree")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        match result {
            Ok(output) => {
                Ok(output.status.success()
                    && String::from_utf8_lossy(&output.stdout).trim() == "true")
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }
}

#[cfg(feature = "git_test")]
pub mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    pub struct GitCommandTest {}

    impl GitCommandTest {
        pub fn new() -> GitCommandTest {
            GitCommandTest {}
        }
    }

    impl GitCommand for GitCommandTest {
        fn init(&self, _path: &Path) -> io::Result<()> {
            Ok(())
        }
        fn commited_ts(&self, _path: &Path) -> io::Result<Option<i64>> {
            Ok(Some(0))
        }
        fn is_git_repository(&self, _path: &Path) -> io::Result<bool> {
            Ok(true)
        }
    }

    impl Default for GitCommandTest {
        fn default() -> Self {
            Self::new()
        }
    }
}
