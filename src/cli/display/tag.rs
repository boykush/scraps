use std::fmt;

use crate::error::{anyhow::Context, CliError, ScrapsResult};
use colored::Colorize;
use comfy_table::{presets::NOTHING, Cell, CellAlignment, Table};
use scraps_libs::{
    model::{base_url::BaseUrl, tag::Tag},
    slugify,
};
use url::Url;

use crate::usecase::build::model::backlinks_map::BacklinksMap;

pub struct DisplayTag {
    tag: Tag,
    url: Option<Url>,
    backlinks_count: usize,
}

impl DisplayTag {
    pub fn new(
        tag: &Tag,
        base_url: Option<&BaseUrl>,
        backlinks_map: &BacklinksMap,
    ) -> ScrapsResult<DisplayTag> {
        let url = base_url
            .map(|base_url| {
                base_url
                    .as_url()
                    .join(&format!("tags/{}.html", tag_slug_path(tag)))
            })
            .transpose()
            .context(CliError::Display)?;
        let backlinks_count = backlinks_map.get_tag(tag).len();

        Ok(DisplayTag {
            tag: tag.clone(),
            url,
            backlinks_count,
        })
    }

    pub fn tag(&self) -> &Tag {
        &self.tag
    }

    pub fn url(&self) -> Option<&Url> {
        self.url.as_ref()
    }

    pub fn backlinks_count(&self) -> usize {
        self.backlinks_count
    }
}

/// Slugify each segment of a hierarchical tag and join with `/`. Used both
/// for URL generation and HTML output paths.
fn tag_slug_path(tag: &Tag) -> String {
    tag.segments()
        .iter()
        .map(|s| slugify::by_dash(s))
        .collect::<Vec<_>>()
        .join("/")
}

pub struct DisplayTagTable {
    tags: Vec<DisplayTag>,
    has_url: bool,
}

impl DisplayTagTable {
    pub fn new(tags: Vec<DisplayTag>) -> Self {
        let has_url = tags.iter().any(|t| t.url().is_some());
        Self { tags, has_url }
    }
}

impl fmt::Display for DisplayTagTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.tags.is_empty() {
            return Ok(());
        }

        let mut table = Table::new();
        table.load_preset(NOTHING);

        if self.has_url {
            table.set_header(vec![
                Cell::new("Tag".bold()),
                Cell::new("Count".bold()),
                Cell::new("URL".bold()),
            ]);
        } else {
            table.set_header(vec![Cell::new("Tag".bold()), Cell::new("Count".bold())]);
        }

        for tag in &self.tags {
            if self.has_url {
                let url_str = tag
                    .url()
                    .map(|u| u.to_string().blue().to_string())
                    .unwrap_or_default();
                table.add_row(vec![
                    Cell::new(tag.tag().to_string()),
                    Cell::new(tag.backlinks_count()).set_alignment(CellAlignment::Right),
                    Cell::new(url_str),
                ]);
            } else {
                table.add_row(vec![
                    Cell::new(tag.tag().to_string()),
                    Cell::new(tag.backlinks_count()).set_alignment(CellAlignment::Right),
                ]);
            }
        }

        write!(f, "{table}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_tag_table_with_url_contains_all_columns() {
        let tags = vec![
            DisplayTag {
                tag: Tag::from("Rust"),
                url: Some(Url::parse("https://example.com/tags/rust.html").unwrap()),
                backlinks_count: 5,
            },
            DisplayTag {
                tag: Tag::from("CLI"),
                url: Some(Url::parse("https://example.com/tags/cli.html").unwrap()),
                backlinks_count: 2,
            },
        ];
        let table = DisplayTagTable::new(tags);
        let output = table.to_string();
        assert!(output.contains("Tag"));
        assert!(output.contains("Count"));
        assert!(output.contains("URL"));
        assert!(output.contains("Rust"));
        assert!(output.contains("CLI"));
    }

    #[test]
    fn display_tag_table_without_url_omits_url_column() {
        let tags = vec![DisplayTag {
            tag: Tag::from("Rust"),
            url: None,
            backlinks_count: 3,
        }];
        let table = DisplayTagTable::new(tags);
        let output = table.to_string();
        assert!(output.contains("Tag"));
        assert!(output.contains("Count"));
        assert!(!output.contains("URL"));
    }

    #[test]
    fn display_tag_table_empty_produces_no_output() {
        let table = DisplayTagTable::new(vec![]);
        let output = table.to_string();
        assert!(output.is_empty());
    }

    #[test]
    fn display_tag_table_renders_hierarchical_path() {
        // Hierarchical tag like ai/ml should display its full path in the
        // "Tag" column, not just the leaf segment.
        let tags = vec![DisplayTag {
            tag: Tag::from("ai/ml"),
            url: Some(Url::parse("https://example.com/tags/ai/ml.html").unwrap()),
            backlinks_count: 1,
        }];
        let table = DisplayTagTable::new(tags);
        let output = table.to_string();
        assert!(output.contains("ai/ml"));
    }
}
