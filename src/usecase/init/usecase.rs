use crate::constants::CONFIG_FILE_NAME;
use crate::error::{anyhow::Context, InitError, ScrapsResult};
use std::{fs, path::Path};

pub struct InitUsecase;

impl InitUsecase {
    pub fn new() -> InitUsecase {
        InitUsecase
    }

    pub fn execute(&self, project_dir: &Path) -> ScrapsResult<()> {
        let config_toml_file = project_dir.join(CONFIG_FILE_NAME);
        fs::write(&config_toml_file, include_str!("builtins/.scraps.toml"))
            .context(InitError::WriteFailure(config_toml_file))?;

        let gitignore_file = project_dir.join(".gitignore");
        fs::write(&gitignore_file, "_site\n").context(InitError::WriteFailure(gitignore_file))
    }
}

impl Default for InitUsecase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::test_fixtures::{simple_temp_dir, SimpleTempDir};
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn it_writes_config_and_gitignore(#[from(simple_temp_dir)] temp_dir: SimpleTempDir) {
        let usecase = InitUsecase::new();

        usecase.execute(&temp_dir.path).unwrap();

        let config_path = temp_dir.path.join(CONFIG_FILE_NAME);
        assert!(config_path.exists());
        let config_contents = std::fs::read_to_string(&config_path).unwrap();
        assert!(config_contents.contains("[ssg]"));

        let gitignore_path = temp_dir.path.join(".gitignore");
        assert!(gitignore_path.exists());
        let gitignore_contents = std::fs::read_to_string(&gitignore_path).unwrap();
        assert_eq!(gitignore_contents, "_site\n");

        assert!(!temp_dir.path.join("scraps").exists());
        assert!(!temp_dir.path.join(".git").exists());
    }
}
