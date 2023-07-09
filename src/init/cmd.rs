use std::{fs, path::PathBuf};

use anyhow::Context;

use crate::libs::error::{error::ScrapError, result::ScrapResult};

pub struct InitCommand {}

impl InitCommand {
    pub fn new() -> InitCommand {
        InitCommand {}
    }

    pub fn run(&self, project_name: &str) -> ScrapResult<()> {
        let project_dir = PathBuf::from(format!("./{}", project_name));
        let scraps_dir = project_dir.join("scraps");
        let config_toml_file = project_dir.join("Config.toml");
        let gitignore_file = project_dir.join(".gitignore");

        fs::create_dir_all(&project_dir).context(ScrapError::FileWriteError)?;
        fs::create_dir(&scraps_dir).context(ScrapError::FileWriteError)?;
        fs::write(&config_toml_file, "title = \"\"").context(ScrapError::FileWriteError)?;
        fs::write(&gitignore_file, "public").context(ScrapError::FileWriteError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_run() {
        let project_path = PathBuf::from("tests/resource/init/cmd/it_run");

        let command = InitCommand::new();

        let result = command.run(project_path.to_str().unwrap());
        assert!(result.is_ok());

        assert!(project_path.exists());
        assert!(project_path.join("scraps").exists());
        assert!(project_path.join("Config.toml").exists());
        assert!(project_path.join(".gitignore").exists());

        fs::remove_dir_all(&project_path).unwrap()
    }
}
