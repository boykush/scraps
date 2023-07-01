use std::{fs, path::PathBuf};

use anyhow::Context;

use crate::libs::{
    error::{error::ScrapError, result::ScrapResult},
    git::GitCommand,
};

pub struct InitCommand {
    project_name: String,
    git_command: Box<dyn GitCommand>,
}

impl InitCommand {
    pub fn new(project_name: &str, git_command: Box<dyn GitCommand>) -> InitCommand {
        InitCommand {
            project_name: project_name.to_string(),
            git_command: git_command,
        }
    }

    pub fn run(&self) -> ScrapResult<()> {
        let project_dir = PathBuf::from(format!("./{}", &self.project_name));
        let scraps_dir = project_dir.join("scraps");
        let config_toml_file = project_dir.join("Config.toml");
        let gitignore_file = project_dir.join(".gitignore");

        fs::create_dir_all(&project_dir).context(ScrapError::FileWriteError)?;
        fs::create_dir(&scraps_dir).context(ScrapError::FileWriteError)?;
        fs::write(&config_toml_file, "title = \"\"").context(ScrapError::FileWriteError)?;
        fs::write(&gitignore_file, "public").context(ScrapError::FileWriteError)?;
        self.git_command.init(&project_dir)
    }
}

#[cfg(test)]
mod tests {
    use crate::libs::git::GitCommandImpl;

    use super::*;

    #[test]
    fn it_run() {
        let project_name = PathBuf::from("tests/resource/init/cmd/it_run");
        let git_command = GitCommandImpl::new();

        let command = InitCommand::new(project_name.to_str().unwrap(), Box::new(git_command));

        let result = command.run();
        assert!(result.is_ok());

        assert!(project_name.exists());
        assert!(project_name.join("scraps").exists());
        assert!(project_name.join("Config.toml").exists());
        assert!(project_name.join(".gitignore").exists());
        assert!(project_name.join(".git").exists());

        fs::remove_dir_all(&project_name).unwrap()
    }
}
