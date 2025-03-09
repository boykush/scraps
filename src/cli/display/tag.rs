use std::fmt;

use colored::Colorize;
use itertools::Itertools;
use scraps_libs::error::{anyhow::Context, ScrapsResult, ScrapsError};
use scraps_libs::model::{tag::Tag, title::Title};
use url::Url;

use crate::build::model::linked_scraps_map::LinkedScrapsMap;

pub struct DisplayTag {
    title: Title,
    url: Url,
    linked_count: usize,
}

impl DisplayTag {
    pub fn new(
        tag: &Tag,
        base_url: &Url,
        linked_scraps_map: &LinkedScrapsMap,
    ) -> ScrapsResult<DisplayTag> {
        let url = base_url
            .join(&format!("scraps/{}.html", tag.title.slug))
            .context(ScrapsError::CliDisplay)?;
        let linked_count = linked_scraps_map.linked_by(&tag.title).len();

        Ok(DisplayTag {
            title: tag.title.to_owned(),
            url,
            linked_count,
        })
    }

    pub fn linked_count(&self) -> usize {
        self.linked_count
    }
}

impl fmt::Display for DisplayTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let title_with_linked_count_str = format!("{}({})", self.title, self.linked_count).bold();
        let url_str = self.url.to_string().blue();

        let tag_str = vec![title_with_linked_count_str, url_str]
            .into_iter()
            .map(|c| c.to_string())
            .collect_vec()
            .join(" ");

        write!(f, "{}", tag_str)
    }
}
