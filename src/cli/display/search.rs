use std::fmt;

use colored::Colorize;
use itertools::Itertools;
use scraps_libs::search::result::SearchResult;

pub struct DisplaySearch {
    title: String,
    url: String,
}

impl DisplaySearch {
    pub fn new(search_result: &SearchResult) -> Self {
        DisplaySearch {
            title: search_result.title.clone(),
            url: search_result.url.clone(),
        }
    }
}

impl fmt::Display for DisplaySearch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let title_str = self.title.bold();
        let url_str = self.url.blue();

        let search_str = vec![title_str, url_str]
            .into_iter()
            .map(|c| c.to_string())
            .collect_vec()
            .join(" ");

        write!(f, "{search_str}")
    }
}
