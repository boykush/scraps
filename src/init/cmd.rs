use std::{fs, path::PathBuf};

use scraps_libs::{
    error::{anyhow::Context, ScrapResult, ScrapsError},
    git::GitCommand,
};

pub struct InitCommand<GC: GitCommand> {
    git_command: GC,
}

impl<GC: GitCommand> InitCommand<GC> {
    pub fn new(git_command: GC) -> InitCommand<GC> {
        InitCommand { git_command }
    }

    pub fn run(&self, project_name: &str) -> ScrapResult<()> {
        let project_dir = &PathBuf::from(format!("./{project_name}"));
        let scraps_dir = project_dir.join("scraps");
        let config_toml_file = project_dir.join("Config.toml");
        let gitignore_file = project_dir.join(".gitignore");

        fs::create_dir_all(project_dir).context(ScrapsError::FileWrite)?;
        fs::create_dir(scraps_dir).context(ScrapsError::FileWrite)?;
        fs::write(config_toml_file, include_str!("builtins/Config.toml"))
            .context(ScrapsError::FileWrite)?;
        fs::write(gitignore_file, "public").context(ScrapsError::FileWrite)?;
        self.git_command.init(project_dir)
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

        let result = command.run(project_path.to_str().unwrap());
        assert!(result.is_ok());

        assert!(project_path.exists());
        assert!(project_path.join("scraps").exists());
        assert!(project_path.join("Config.toml").exists());
        assert!(project_path.join(".gitignore").exists());
        assert!(project_path.join(".git").exists());

        fs::remove_dir_all(&project_path).unwrap()
    }
}
