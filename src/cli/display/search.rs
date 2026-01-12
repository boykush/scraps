use std::fmt;
use std::path::{Path, PathBuf};

use crate::usecase::search::usecase::SearchResult;
use colored::Colorize;
use url::Url;

pub struct DisplaySearch {
    title: String,
    file_path: PathBuf,
}

impl DisplaySearch {
    pub fn new_with_file_path(search_result: &SearchResult, file_path: &Path) -> Self {
        DisplaySearch {
            title: search_result.title.to_string(),
            file_path: file_path.to_path_buf(),
        }
    }
}

impl fmt::Display for DisplaySearch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let title_str = self.title.bold();

        // Convert file path to properly encoded file:// URL
        // This handles spaces and special characters correctly
        let file_url_str = if let Ok(url) = Url::from_file_path(&self.file_path) {
            url.to_string().blue()
        } else {
            // Fallback to display if URL conversion fails
            self.file_path.display().to_string().blue()
        };

        write!(f, "{title_str} {file_url_str}")
    }
}
