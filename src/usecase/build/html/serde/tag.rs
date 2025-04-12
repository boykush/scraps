use crate::usecase::build::model::backlinks_map::BacklinksMap;
use scraps_libs::model::{slug::Slug, tag::Tag};

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct TagTera {
    title: String,
    slug: String,
    pub linked_count: usize,
}

impl TagTera {
    pub fn new(tag: &Tag, linked_scraps_map: &BacklinksMap) -> TagTera {
        let linked_count = linked_scraps_map.get(&tag.title.clone().into()).len();
        TagTera {
            title: tag.title.to_string(),
            slug: Slug::from(tag.title.clone()).to_string(),
            linked_count,
        }
    }
}
