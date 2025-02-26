use itertools::Itertools;

use crate::build::model::linked_scraps_map::LinkedScrapsMap;
use scraps_libs::model::tags::Tags;

use super::tag::TagTera;

#[derive(serde::Serialize, PartialEq, Debug)]
pub struct TagsTera(Vec<TagTera>);

impl TagsTera {
    pub fn new(tags: &Tags, linked_scraps_map: &LinkedScrapsMap) -> TagsTera {
        let stags = tags
            .values
            .iter()
            .map(|tag| TagTera::new(tag, linked_scraps_map));
        let sorted = stags.sorted_by_key(|s| s.linked_count).rev();
        TagsTera(sorted.collect_vec())
    }
}
