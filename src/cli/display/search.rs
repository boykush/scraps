use std::fmt;

use crate::usecase::search::usecase::SearchResult;
use colored::Colorize;

pub struct DisplaySearch {
    title: String,
    url: Option<String>,
}

impl DisplaySearch {
    pub fn new(search_result: &SearchResult) -> Self {
        DisplaySearch {
            title: search_result.title.to_string(),
            url: search_result.url.clone(),
        }
    }
}

impl fmt::Display for DisplaySearch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let title_str = self.title.bold();

        let search_str = if let Some(url) = &self.url {
            let url_str = url.blue();
            format!("{title_str} {url_str}")
        } else {
            format!("{title_str}")
        };

        write!(f, "{search_str}")
    }
}
