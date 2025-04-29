use crate::error::{anyhow::Context, InitError, ScrapsResult};
use std::{fs, path::PathBuf};

use scraps_libs::git::GitCommand;

pub struct InitCommand<GC: GitCommand> {
    git_command: GC,
}

impl<GC: GitCommand> InitCommand<GC> {
    pub fn new(git_command: GC) -> InitCommand<GC> {
        InitCommand { git_command }
    }

    pub fn run(&self, project_name: &str) -> ScrapsResult<()> {
        let project_dir = &PathBuf::from(format!("./{project_name}"));
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

    use super::*;

    #[test]
    fn it_run() {
        let git_command = GitCommandImpl::new();
        let project_path = PathBuf::from("tests/resource/init/cmd/it_run");

        let command = InitCommand::new(git_command);

        command.run(project_path.to_str().unwrap()).unwrap();

        assert!(project_path.exists());
        assert!(project_path.join("scraps").exists());
        assert!(project_path.join("Config.toml").exists());
        assert!(project_path.join(".gitignore").exists());
        assert!(project_path.join(".git").exists());

        fs::remove_dir_all(&project_path).unwrap()
    }
}
