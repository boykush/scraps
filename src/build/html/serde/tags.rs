use itertools::Itertools;

use crate::build::model::{linked_scraps_map::LinkedScrapsMap, tag::Tag};

use super::tag::SerializeTag;

#[derive(serde::Serialize, PartialEq, Debug)]
pub struct SerializeTags(Vec<SerializeTag>);

impl SerializeTags {
    pub fn new(tags: &Vec<Tag>, linked_scraps_map: &LinkedScrapsMap) -> SerializeTags {
        let stags = tags
            .iter()
            .map(|tag| SerializeTag::new(tag, linked_scraps_map));
        let sorted = stags.sorted_by_key(|s| s.linked_count).rev();
        SerializeTags(sorted.collect_vec())
    }
}
