use crate::build::model::linked_scraps_map::LinkedScrapsMap;
use scraps_libs::model::tag::Tag;

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct TagTera {
    title: String,
    slug: String,
    pub linked_count: usize,
}

impl TagTera {
    pub fn new(tag: &Tag, linked_scraps_map: &LinkedScrapsMap) -> TagTera {
        let linked_count = linked_scraps_map.linked_by(&tag.title).len();
        TagTera {
            title: tag.title.to_string(),
            slug: tag.title.slug.to_string(),
            linked_count,
        }
    }
}
