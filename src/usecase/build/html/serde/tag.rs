use crate::usecase::build::model::backlinks_map::BacklinksMap;
use scraps_libs::model::{page_path::PagePath, tag::Tag};

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct TagTera {
    /// Full hierarchical tag path for display, e.g. `ai/ml`.
    pub title: String,
    /// Page URL relative to `base_url`, already percent-encoded.
    url_path: String,
    pub backlinks_count: usize,
}

impl TagTera {
    pub fn new(tag: &Tag, backlinks_map: &BacklinksMap) -> TagTera {
        let backlinks_count = backlinks_map.get_tag(tag).len();
        TagTera {
            title: tag.to_string(),
            url_path: PagePath::of_tag(tag).to_url_path(),
            backlinks_count,
        }
    }
}
