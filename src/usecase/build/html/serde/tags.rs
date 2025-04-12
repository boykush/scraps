use itertools::Itertools;

use crate::usecase::build::model::backlinks_map::BacklinksMap;
use scraps_libs::model::tags::Tags;

use super::tag::TagTera;

#[derive(serde::Serialize, PartialEq, Debug)]
pub struct TagsTera(Vec<TagTera>);

impl TagsTera {
    pub fn new(tags: &Tags, backlinks_map: &BacklinksMap) -> TagsTera {
        let stags = tags
            .clone()
            .into_iter()
            .map(|tag| TagTera::new(&tag, backlinks_map));
        let sorted = stags.sorted_by_key(|s| s.backlinks_count).rev();
        TagsTera(sorted.collect_vec())
    }
}
