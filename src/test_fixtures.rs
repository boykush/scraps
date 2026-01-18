//! Test fixtures for scraps tests using rstest and tempfile
//!
//! This module provides composable test fixtures that create temporary
//! directory structures with automatic cleanup.

use rstest::fixture;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// High-level fixture for a complete Scraps project structure
///
/// Provides a temporary project with scraps/, static/, public/, and templates/
/// directories automatically created and cleaned up after the test.
///
/// # Example
/// ```no_run
/// use rstest::rstest;
/// use crate::test_fixtures::TempScrapProject;
///
/// #[rstest]
/// fn test_build(temp_scrap_project: TempScrapProject) {
///     temp_scrap_project
///         .add_scrap("test.md", b"# Test Content");
///
///     // Use temp_scrap_project.scraps_dir, .static_dir, .public_dir, etc.
///     // Automatic cleanup when temp_scrap_project goes out of scope
/// }
/// ```
pub struct TempScrapProject {
    #[allow(dead_code)]
    temp_dir: TempDir,
    pub scraps_dir: PathBuf,
    pub static_dir: PathBuf,
    pub public_dir: PathBuf,
    pub templates_dir: PathBuf,
    pub project_root: PathBuf,
}

impl TempScrapProject {
    /// Create a new temporary Scraps project with standard directory structure
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_root = temp_dir.path().to_path_buf();

        let scraps_dir = project_root.join("scraps");
        let static_dir = project_root.join("static");
        let public_dir = project_root.join("public");
        let templates_dir = project_root.join("templates");

        // Create all directories
        fs::create_dir_all(&scraps_dir).expect("Failed to create scraps dir");
        fs::create_dir_all(&static_dir).expect("Failed to create static dir");
        fs::create_dir_all(&public_dir).expect("Failed to create public dir");
        fs::create_dir_all(&templates_dir).expect("Failed to create templates dir");

        Self {
            temp_dir,
            scraps_dir,
            static_dir,
            public_dir,
            templates_dir,
            project_root,
        }
    }

    /// Add a markdown scrap file to the scraps directory
    ///
    /// # Arguments
    /// * `filename` - Relative path from scraps_dir (e.g., "test.md" or "subdir/test.md")
    /// * `content` - File content as bytes
    ///
    /// # Example
    /// ```no_run
    /// project.add_scrap("test.md", b"# Header\n\nContent");
    /// ```
    pub fn add_scrap(&self, filename: &str, content: &[u8]) -> &Self {
        let path = self.scraps_dir.join(filename);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent dir");
        }
        fs::write(&path, content).expect("Failed to write scrap file");
        self
    }

    /// Add a scrap file in a context subdirectory
    ///
    /// # Arguments
    /// * `context` - Context directory name (e.g., "Context")
    /// * `filename` - Filename within the context directory
    /// * `content` - File content as bytes
    ///
    /// # Example
    /// ```no_run
    /// project.add_scrap_with_context("Context", "test.md", b"# Contextual Content");
    /// ```
    pub fn add_scrap_with_context(&self, context: &str, filename: &str, content: &[u8]) -> &Self {
        let context_dir = self.scraps_dir.join(context);
        fs::create_dir_all(&context_dir).expect("Failed to create context dir");
        let path = context_dir.join(filename);
        fs::write(&path, content).expect("Failed to write scrap file");
        self
    }

    /// Add a Config.toml file to the project root
    ///
    /// # Arguments
    /// * `content` - TOML configuration content as bytes
    ///
    /// # Example
    /// ```no_run
    /// project.add_config(b"scraps_dir = \"docs\"\n\n[ssg]\nbase_url = \"https://example.com/\"\ntitle = \"Test\"");
    /// ```
    pub fn add_config(&self, content: &[u8]) -> &Self {
        let config_path = self.project_root.join("Config.toml");
        fs::write(&config_path, content).expect("Failed to write config file");
        self
    }

    /// Add a template file to the templates directory
    ///
    /// # Arguments
    /// * `filename` - Template filename (e.g., "template.md")
    /// * `content` - Template content as bytes
    pub fn add_template(&self, filename: &str, content: &[u8]) -> &Self {
        let path = self.templates_dir.join(filename);
        fs::write(&path, content).expect("Failed to write template file");
        self
    }

    /// Add a static file to the static directory
    ///
    /// # Arguments
    /// * `filename` - Relative path from static_dir (e.g., "style.css" or "js/app.js")
    /// * `content` - File content as bytes
    pub fn add_static_file(&self, filename: &str, content: &[u8]) -> &Self {
        let path = self.static_dir.join(filename);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent dir");
        }
        fs::write(&path, content).expect("Failed to write static file");
        self
    }

    /// Get the path to a file in the public directory
    ///
    /// Useful for checking generated output files.
    ///
    /// # Arguments
    /// * `filename` - Relative path from public_dir (e.g., "index.html" or "scraps/test.html")
    ///
    /// # Example
    /// ```no_run
    /// assert!(project.public_path("index.html").exists());
    /// ```
    pub fn public_path(&self, filename: &str) -> PathBuf {
        self.public_dir.join(filename)
    }

    /// Get the path to a file in the scraps directory
    ///
    /// # Arguments
    /// * `filename` - Relative path from scraps_dir
    pub fn scrap_path(&self, filename: &str) -> PathBuf {
        self.scraps_dir.join(filename)
    }
}

impl Default for TempScrapProject {
    fn default() -> Self {
        Self::new()
    }
}

/// rstest fixture for TempScrapProject
///
/// Use this in tests with `#[rstest]` annotation.
#[fixture]
pub fn temp_scrap_project() -> TempScrapProject {
    TempScrapProject::new()
}

/// Lightweight temporary directory for simple tests
///
/// Provides a single temporary directory with helper methods for adding
/// files and subdirectories.
///
/// # Example
/// ```no_run
/// use rstest::rstest;
/// use crate::test_fixtures::SimpleTempDir;
///
/// #[rstest]
/// fn test_simple(simple_temp_dir: SimpleTempDir) {
///     simple_temp_dir
///         .add_file("test.txt", b"content")
///         .add_dir("subdir");
///
///     // Use simple_temp_dir.path
/// }
/// ```
pub struct SimpleTempDir {
    #[allow(dead_code)]
    temp_dir: TempDir,
    pub path: PathBuf,
}

impl SimpleTempDir {
    /// Create a new temporary directory
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let path = temp_dir.path().to_path_buf();
        Self { temp_dir, path }
    }

    /// Add a file to the temporary directory
    ///
    /// # Arguments
    /// * `relative_path` - Path relative to the temp directory root
    /// * `content` - File content as bytes
    ///
    /// # Example
    /// ```no_run
    /// temp_dir.add_file("subdir/test.txt", b"content");
    /// ```
    pub fn add_file(&self, relative_path: &str, content: &[u8]) -> &Self {
        let full_path = self.path.join(relative_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent dir");
        }
        fs::write(&full_path, content).expect("Failed to write file");
        self
    }

    /// Add a directory to the temporary directory
    ///
    /// # Arguments
    /// * `relative_path` - Path relative to the temp directory root
    ///
    /// # Example
    /// ```no_run
    /// temp_dir.add_dir("subdir/nested");
    /// ```
    pub fn add_dir(&self, relative_path: &str) -> &Self {
        let full_path = self.path.join(relative_path);
        fs::create_dir_all(&full_path).expect("Failed to create directory");
        self
    }
}

impl Default for SimpleTempDir {
    fn default() -> Self {
        Self::new()
    }
}

/// rstest fixture for SimpleTempDir
///
/// Use this in tests with `#[rstest]` annotation.
#[fixture]
pub fn simple_temp_dir() -> SimpleTempDir {
    SimpleTempDir::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temp_scrap_project_creates_directories() {
        let project = TempScrapProject::new();

        assert!(project.scraps_dir.exists());
        assert!(project.static_dir.exists());
        assert!(project.public_dir.exists());
        assert!(project.templates_dir.exists());
        assert!(project.project_root.exists());
    }

    #[test]
    fn test_temp_scrap_project_add_scrap() {
        let project = TempScrapProject::new();
        project.add_scrap("test.md", b"# Test Content");

        let scrap_path = project.scrap_path("test.md");
        assert!(scrap_path.exists());
        assert_eq!(fs::read_to_string(scrap_path).unwrap(), "# Test Content");
    }

    #[test]
    fn test_temp_scrap_project_add_scrap_with_context() {
        let project = TempScrapProject::new();
        project.add_scrap_with_context("Context", "test.md", b"# Contextual");

        let scrap_path = project.scraps_dir.join("Context/test.md");
        assert!(scrap_path.exists());
        assert_eq!(fs::read_to_string(scrap_path).unwrap(), "# Contextual");
    }

    #[test]
    fn test_temp_scrap_project_add_config() {
        let project = TempScrapProject::new();
        project.add_config(b"[ssg]\nbase_url = \"https://example.com/\"\ntitle = \"Test\"");

        let config_path = project.project_root.join("Config.toml");
        assert!(config_path.exists());
    }

    #[test]
    fn test_simple_temp_dir_creates_directory() {
        let temp_dir = SimpleTempDir::new();
        assert!(temp_dir.path.exists());
    }

    #[test]
    fn test_simple_temp_dir_add_file() {
        let temp_dir = SimpleTempDir::new();
        temp_dir.add_file("test.txt", b"content");

        let file_path = temp_dir.path.join("test.txt");
        assert!(file_path.exists());
        assert_eq!(fs::read_to_string(file_path).unwrap(), "content");
    }

    #[test]
    fn test_simple_temp_dir_add_dir() {
        let temp_dir = SimpleTempDir::new();
        temp_dir.add_dir("subdir");

        let dir_path = temp_dir.path.join("subdir");
        assert!(dir_path.exists());
        assert!(dir_path.is_dir());
    }

    #[test]
    fn test_cleanup_on_drop() {
        let path = {
            let temp_dir = SimpleTempDir::new();
            temp_dir.add_file("test.txt", b"content");
            temp_dir.path.clone()
        };

        // After temp_dir is dropped, the directory should be cleaned up
        assert!(!path.exists());
    }
}
