use crate::build::model::{linked_scraps_map::LinkedScrapsMap, tag::Tag};

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct SerializeTag {
    title: String,
    pub linked_count: usize,
}

impl SerializeTag {
    pub fn new(tag: &Tag, linked_scraps_map: &LinkedScrapsMap) -> SerializeTag {
        let linked_count = linked_scraps_map.linked_by(&tag.title).len();
        SerializeTag {
            title: tag.title.to_string(),
            linked_count: linked_count,
        }
    }
}
