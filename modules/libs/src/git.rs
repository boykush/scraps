use std::{
    io,
    path::Path,
    process::{Command, Stdio},
};

pub trait GitCommand {
    fn init(&self, path: &Path) -> io::Result<()>;
    fn commited_ts(&self, path: &Path) -> io::Result<Option<i64>>;
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
    }

    impl Default for GitCommandTest {
        fn default() -> Self {
            Self::new()
        }
    }
}
