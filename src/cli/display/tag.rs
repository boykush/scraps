use std::fmt;

use anyhow::Context;
use colored::Colorize;
use itertools::Itertools;
use url::Url;

use crate::libs::{
    error::{ScrapError, ScrapResult},
    model::{tag::Tag, title::Title},
};

pub struct DisplayTag {
    title: Title,
    url: Url,
}

impl DisplayTag {
    pub fn new(tag: &Tag, base_url: &Url) -> ScrapResult<DisplayTag> {
        let url = base_url
            .join(&tag.title.slug.to_string())
            .context(ScrapError::CliDisplay)?;

        Ok(DisplayTag {
            title: tag.title.to_owned(),
            url,
        })
    }
}

impl fmt::Display for DisplayTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let title_str = self.title.to_string().bold();
        let url_str = self.url.to_string().blue();

        let tag_str = vec![title_str, url_str]
            .into_iter()
            .map(|c| c.to_string())
            .collect_vec()
            .join(" ");

        write!(f, "{}", tag_str)
    }
}
