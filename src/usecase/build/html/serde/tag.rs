use crate::usecase::build::model::backlinks_map::BacklinksMap;
use scraps_libs::model::{slug::Slug, tag::Tag};

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct TagTera {
    title: String,
    slug: String,
    pub backlinks_count: usize,
}

impl TagTera {
    pub fn new(tag: &Tag, backlinks_map: &BacklinksMap) -> TagTera {
        let backlinks_count = backlinks_map.get(&tag.title.clone().into()).len();
        TagTera {
            title: tag.title.to_string(),
            slug: Slug::from(tag.title.clone()).to_string(),
            backlinks_count,
        }
    }
}
