use std::fmt;

use crate::error::{anyhow::Context, CliError, ScrapsResult};
use colored::Colorize;
use itertools::Itertools;
use scraps_libs::model::{slug::Slug, tag::Tag, title::Title};
use url::Url;

use crate::usecase::build::model::backlinks_map::BacklinksMap;

pub struct DisplayTag {
    title: Title,
    url: Url,
    backlinks_count: usize,
}

impl DisplayTag {
    pub fn new(
        tag: &Tag,
        base_url: &Url,
        backlinks_map: &BacklinksMap,
    ) -> ScrapsResult<DisplayTag> {
        let url = base_url
            .join(&format!("scraps/{}.html", Slug::from(tag.title.clone())))
            .context(CliError::Display)?;
        let backlinks_count = backlinks_map.get(&tag.title.clone().into()).len();

        Ok(DisplayTag {
            title: tag.title.to_owned(),
            url,
            backlinks_count,
        })
    }

    pub fn backlinks_count(&self) -> usize {
        self.backlinks_count
    }
}

impl fmt::Display for DisplayTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let title_with_backlinks_count_str =
            format!("{}({})", self.title, self.backlinks_count).bold();
        let url_str = self.url.to_string().blue();

        let tag_str = vec![title_with_backlinks_count_str, url_str]
            .into_iter()
            .map(|c| c.to_string())
            .collect_vec()
            .join(" ");

        write!(f, "{tag_str}")
    }
}
