use crate::cli::config::scrap_config::ScrapConfig;
use crate::error::ScrapsResult;
use anyhow::anyhow;
use std::env;
use std::path::{Path, PathBuf};

/// Resolves project paths for scraps commands
pub struct PathResolver {
    project_root: PathBuf,
}

impl PathResolver {
    /// Create a new PathResolver
    /// If project_path is None, uses current directory
    pub fn new(project_path: Option<&Path>) -> ScrapsResult<Self> {
        let project_root = match project_path {
            Some(path) => {
                let absolute_path = if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    env::current_dir()?.join(path)
                };

                if !absolute_path.exists() {
                    return Err(anyhow!(
                        "Project directory does not exist: {}",
                        absolute_path.display()
                    ));
                }

                if !absolute_path.is_dir() {
                    return Err(anyhow!(
                        "Path is not a directory: {}",
                        absolute_path.display()
                    ));
                }

                absolute_path
            }
            None => env::current_dir()?,
        };

        Ok(PathResolver { project_root })
    }

    /// Get the scraps directory path
    pub fn scraps_dir(&self, config: &ScrapConfig) -> PathBuf {
        match &config.scraps_dir {
            Some(dir) => self.project_root.join(dir),
            None => self.project_root.join("scraps"),
        }
    }

    /// Get the static directory path
    pub fn static_dir(&self) -> PathBuf {
        self.project_root.join("static")
    }

    /// Get the public directory path
    pub fn public_dir(&self) -> PathBuf {
        self.project_root.join("public")
    }

    /// Get the templates directory path
    pub fn templates_dir(&self) -> PathBuf {
        self.project_root.join("templates")
    }

    /// Get the config file path (Config.toml)
    pub fn config_path(&self) -> PathBuf {
        self.project_root.join("Config.toml")
    }

    /// Get the project root directory
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{simple_temp_dir, SimpleTempDir};
    use rstest::rstest;
    use std::env;

    #[test]
    fn test_new_with_current_directory() {
        let resolver = PathResolver::new(None).unwrap();

        let expected_root = env::current_dir().unwrap();
        assert_eq!(resolver.project_root(), expected_root.as_path());
    }

    #[rstest]
    fn test_new_with_specified_path(#[from(simple_temp_dir)] temp_dir: SimpleTempDir) {
        temp_dir.add_dir("test_project_new");

        let test_project_path = temp_dir.path.join("test_project_new");
        let resolver = PathResolver::new(Some(&test_project_path)).unwrap();
        assert_eq!(
            resolver.project_root().file_name().unwrap(),
            "test_project_new"
        );
    }

    #[test]
    fn test_new_with_nonexistent_path() {
        let nonexistent_path = PathBuf::from("nonexistent_directory");
        let result = PathResolver::new(Some(&nonexistent_path));
        assert!(result.is_err());
    }

    #[rstest]
    fn test_scraps_dir_path_default(#[from(simple_temp_dir)] temp_dir: SimpleTempDir) {
        temp_dir.add_dir("test_project_scraps").add_file(
            "test_project_scraps/Config.toml",
            br#"
title = "Test"
base_url = "http://example.com/"
"#,
        );

        let test_project_path = temp_dir.path.join("test_project_scraps");
        let resolver = PathResolver::new(Some(&test_project_path)).unwrap();
        let config = ScrapConfig::from_path(Some(&test_project_path)).unwrap();

        let scraps_dir = resolver.scraps_dir(&config);
        assert_eq!(scraps_dir.file_name().unwrap(), "scraps");
        assert!(scraps_dir.starts_with(&test_project_path));
    }

    #[rstest]
    fn test_scraps_dir_path_custom(#[from(simple_temp_dir)] temp_dir: SimpleTempDir) {
        temp_dir.add_dir("test_project_scraps_custom").add_file(
            "test_project_scraps_custom/Config.toml",
            br#"
title = "Test"
base_url = "http://example.com/"
scraps_dir = "custom_docs"
"#,
        );

        let test_project_path = temp_dir.path.join("test_project_scraps_custom");
        let resolver = PathResolver::new(Some(&test_project_path)).unwrap();
        let config = ScrapConfig::from_path(Some(&test_project_path)).unwrap();

        let scraps_dir = resolver.scraps_dir(&config);
        assert_eq!(scraps_dir.file_name().unwrap(), "custom_docs");
        assert!(scraps_dir.starts_with(&test_project_path));
    }

    #[rstest]
    fn test_static_dir_path(#[from(simple_temp_dir)] temp_dir: SimpleTempDir) {
        temp_dir.add_dir("test_project_static");

        let test_project_path = temp_dir.path.join("test_project_static");
        let resolver = PathResolver::new(Some(&test_project_path)).unwrap();
        let static_dir = resolver.static_dir();
        assert_eq!(static_dir.file_name().unwrap(), "static");
        assert!(static_dir.starts_with(&test_project_path));
    }

    #[rstest]
    fn test_public_dir_path(#[from(simple_temp_dir)] temp_dir: SimpleTempDir) {
        temp_dir.add_dir("test_project_public");

        let test_project_path = temp_dir.path.join("test_project_public");
        let resolver = PathResolver::new(Some(&test_project_path)).unwrap();
        let public_dir = resolver.public_dir();
        assert_eq!(public_dir.file_name().unwrap(), "public");
        assert!(public_dir.starts_with(&test_project_path));
    }

    #[rstest]
    fn test_templates_dir_path(#[from(simple_temp_dir)] temp_dir: SimpleTempDir) {
        temp_dir.add_dir("test_project_templates");

        let test_project_path = temp_dir.path.join("test_project_templates");
        let resolver = PathResolver::new(Some(&test_project_path)).unwrap();
        let templates_dir = resolver.templates_dir();
        assert_eq!(templates_dir.file_name().unwrap(), "templates");
        assert!(templates_dir.starts_with(&test_project_path));
    }

    #[rstest]
    fn test_config_path(#[from(simple_temp_dir)] temp_dir: SimpleTempDir) {
        temp_dir.add_dir("test_project_config");

        let test_project_path = temp_dir.path.join("test_project_config");
        let resolver = PathResolver::new(Some(&test_project_path)).unwrap();
        let config_path = resolver.config_path();
        assert_eq!(config_path.file_name().unwrap(), "Config.toml");
        assert!(config_path.starts_with(&test_project_path));
    }
}
