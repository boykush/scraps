use itertools::Itertools;

use crate::usecase::build::model::backlinks_map::BacklinksMap;
use scraps_libs::model::tags::Tags;

use super::tag::TagTera;

#[derive(serde::Serialize, PartialEq, Debug)]
pub struct TagsTera(Vec<TagTera>);

impl TagsTera {
    pub fn new(tags: &Tags, linked_scraps_map: &BacklinksMap) -> TagsTera {
        let stags = tags
            .clone()
            .into_iter()
            .map(|tag| TagTera::new(&tag, linked_scraps_map));
        let sorted = stags.sorted_by_key(|s| s.linked_count).rev();
        TagsTera(sorted.collect_vec())
    }
}
