use crate::error::{anyhow::Context, InitError, ScrapsResult};
use std::{fs, path::Path};

use scraps_libs::git::GitCommand;

pub struct InitUsecase<GC: GitCommand> {
    git_command: GC,
}

impl<GC: GitCommand> InitUsecase<GC> {
    pub fn new(git_command: GC) -> InitUsecase<GC> {
        InitUsecase { git_command }
    }

    pub fn execute(&self, project_dir: &Path) -> ScrapsResult<()> {
        let scraps_dir = project_dir.join("scraps");
        let config_toml_file = &project_dir.join("Config.toml");
        let gitignore_file = &project_dir.join(".gitignore");

        fs::create_dir_all(project_dir).context(InitError::CreateDirectory)?;
        fs::create_dir(scraps_dir).context(InitError::CreateDirectory)?;
        fs::write(config_toml_file, include_str!("builtins/Config.toml"))
            .context(InitError::WriteFailure(config_toml_file.clone()))?;
        fs::write(gitignore_file, "public")
            .context(InitError::WriteFailure(gitignore_file.clone()))?;
        self.git_command
            .init(project_dir)
            .context(InitError::GitInit)
    }
}

#[cfg(test)]
mod tests {
    use scraps_libs::git::GitCommandImpl;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn it_run() {
        let git_command = GitCommandImpl::new();
        let project_path = PathBuf::from("tests/resource/init/cmd/it_run");

        let usecase = InitUsecase::new(git_command);

        usecase.execute(&project_path).unwrap();

        assert!(project_path.exists());
        assert!(project_path.join("scraps").exists());
        assert!(project_path.join("Config.toml").exists());
        assert!(project_path.join(".gitignore").exists());
        assert!(project_path.join(".git").exists());

        fs::remove_dir_all(&project_path).unwrap()
    }
}
