use crate::usecase::build::model::backlinks_map::BacklinksMap;
use scraps_libs::{model::tag::Tag, slugify};

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct TagTera {
    /// Full hierarchical tag path for display, e.g. `ai/ml`.
    title: String,
    /// Slug-form path with each segment slugified, joined by `/`.
    /// Used to build the tag's HTML URL: `/tags/<slug>.html`.
    slug: String,
    pub backlinks_count: usize,
}

impl TagTera {
    pub fn new(tag: &Tag, backlinks_map: &BacklinksMap) -> TagTera {
        let backlinks_count = backlinks_map.get_tag(tag).len();
        let slug = tag
            .segments()
            .iter()
            .map(|s| slugify::by_dash(s))
            .collect::<Vec<_>>()
            .join("/");
        TagTera {
            title: tag.to_string(),
            slug,
            backlinks_count,
        }
    }
}
