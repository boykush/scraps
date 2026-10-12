use std::collections::BTreeMap;
use std::fmt;

use scraps_libs::model::{file::ScrapFileStem, key::ScrapKey, scrap::Scrap};

/// The directory under the output root that holds one page per scrap.
pub const SCRAPS_DIR: &str = "scraps";

/// The all-scraps listing, written into `SCRAPS_DIR` beside the scrap pages.
pub const SCRAPS_INDEX_FILE: &str = "index.html";

/// Pages the site writes into `SCRAPS_DIR` that are not a scrap's own.
const GENERATED_IN_SCRAPS_DIR: [(&str, &str); 1] = [(SCRAPS_INDEX_FILE, "the all-scraps listing")];

/// A scrap's page, relative to `SCRAPS_DIR`.
pub fn scrap_page_file(key: &ScrapKey) -> String {
    format!("{}.html", ScrapFileStem::from(key.clone()))
}

/// A scrap's source file, relative to the wiki root.
pub fn scrap_relative_path(scrap: &Scrap) -> String {
    match scrap.ctx() {
        Some(ctx) => format!("{}/{}.md", ctx, scrap.title()),
        None => format!("{}.md", scrap.title()),
    }
}

/// One output path that more than one page would be written to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathCollision {
    /// Relative to the output root.
    pub output_path: String,
    /// Source files of the scraps mapped to the path, relative to the wiki root.
    pub sources: Vec<String>,
    /// The generated page that owns the path, when one does.
    pub generated: Option<&'static str>,
}

/// Every contested output path of a wiki, in path order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathCollisions(Vec<PathCollision>);

impl PathCollisions {
    pub fn find<'a>(scraps: impl IntoIterator<Item = &'a Scrap>) -> PathCollisions {
        let mut sources_by_file: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for scrap in scraps {
            sources_by_file
                .entry(scrap_page_file(&scrap.self_key()))
                .or_default()
                .push(scrap_relative_path(scrap));
        }

        let collisions = sources_by_file
            .into_iter()
            .filter_map(|(file, mut sources)| {
                let generated = GENERATED_IN_SCRAPS_DIR
                    .iter()
                    .find(|(generated_file, _)| *generated_file == file)
                    .map(|(_, page)| *page);
                if sources.len() < 2 && generated.is_none() {
                    return None;
                }
                sources.sort();
                Some(PathCollision {
                    output_path: format!("{SCRAPS_DIR}/{file}"),
                    sources,
                    generated,
                })
            })
            .collect();
        PathCollisions(collisions)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn as_slice(&self) -> &[PathCollision] {
        &self.0
    }
}

impl fmt::Display for PathCollisions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let paths = match self.0.len() {
            1 => "1 output path would be".to_string(),
            n => format!("{n} output paths would each be"),
        };
        write!(f, "{paths} written by more than one page")?;
        for collision in &self.0 {
            write!(f, "\n  {}", collision.output_path)?;
            for source in &collision.sources {
                write!(f, "\n    --> {source}")?;
            }
            if let Some(page) = collision.generated {
                write!(f, "\n    --> {page} (generated)")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scrap(title: &str, ctx: Option<&str>) -> Scrap {
        Scrap::new(title, &ctx.map(Into::into), "")
    }

    #[test]
    fn no_collision_in_a_wiki_of_distinct_paths() {
        let scraps = vec![
            scrap("a b", None),
            scrap("a b", Some("Ctx")),
            scrap("index", Some("Ctx")),
            scrap("c", None),
        ];
        assert!(PathCollisions::find(&scraps).is_empty());
    }

    #[test]
    fn scraps_slugified_to_one_path_collide() {
        let scraps = vec![scrap("a-b", None), scrap("c", None), scrap("a b", None)];
        assert_eq!(
            PathCollisions::find(&scraps).as_slice(),
            &[PathCollision {
                output_path: "scraps/a-b.html".to_string(),
                sources: vec!["a b.md".to_string(), "a-b.md".to_string()],
                generated: None,
            }]
        );
    }

    #[test]
    fn scraps_differing_only_in_case_collide() {
        let scraps = vec![scrap("post", Some("Blog")), scrap("Post", Some("blog"))];
        let collisions = PathCollisions::find(&scraps);
        assert_eq!(collisions.as_slice().len(), 1);
        assert_eq!(
            collisions.as_slice()[0].output_path,
            "scraps/blog/post.html"
        );
        assert_eq!(
            collisions.as_slice()[0].sources,
            vec!["Blog/post.md", "blog/Post.md"]
        );
    }

    #[test]
    fn root_index_scrap_collides_with_the_all_scraps_listing() {
        let scraps = vec![scrap("Index", None)];
        assert_eq!(
            PathCollisions::find(&scraps).as_slice(),
            &[PathCollision {
                output_path: "scraps/index.html".to_string(),
                sources: vec!["Index.md".to_string()],
                generated: Some("the all-scraps listing"),
            }]
        );
    }

    #[test]
    fn display_reports_every_collision_with_its_sources() {
        let scraps = vec![scrap("a b", None), scrap("a-b", None), scrap("index", None)];
        assert_eq!(
            PathCollisions::find(&scraps).to_string(),
            concat!(
                "2 output paths would each be written by more than one page\n",
                "  scraps/a-b.html\n",
                "    --> a b.md\n",
                "    --> a-b.md\n",
                "  scraps/index.html\n",
                "    --> index.md\n",
                "    --> the all-scraps listing (generated)",
            )
        );
    }
}
