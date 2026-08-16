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
        // Title breaks ties so the order is total: index pages slice the top-N
        // tags, and an arbitrary tie-break would reshuffle them between builds.
        let sorted = stags.sorted_by(|a, b| {
            b.backlinks_count
                .cmp(&a.backlinks_count)
                .then_with(|| a.title.cmp(&b.title))
        });
        TagsTera(sorted.collect_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraps_libs::model::scrap::Scrap;

    fn titles(stags: &TagsTera) -> Vec<&str> {
        stags.0.iter().map(|s| s.title.as_str()).collect()
    }

    fn tags_tera(scraps: &[Scrap]) -> TagsTera {
        TagsTera::new(&Tags::new(scraps), &BacklinksMap::new(scraps))
    }

    #[test]
    fn it_orders_by_backlinks_count_desc_then_title_asc() {
        // `delta`..`alpha` all tie at 1 backlink and are declared in reverse
        // alphabetical order, so insertion order cannot be what breaks the tie.
        let scraps = [
            Scrap::new("s1", &None, "#[[popular]] #[[delta]]"),
            Scrap::new("s2", &None, "#[[popular]] #[[charlie]]"),
            Scrap::new("s3", &None, "#[[popular]] #[[bravo]]"),
            Scrap::new("s4", &None, "#[[alpha]]"),
        ];

        assert_eq!(
            titles(&tags_tera(&scraps)),
            vec!["popular", "alpha", "bravo", "charlie", "delta"]
        );
    }

    #[test]
    fn it_orders_tied_tags_the_same_way_on_every_build() {
        // Regression: `Tags` used to be a HashSet, whose per-instance random
        // seed made tied tags swap places between builds and produced noisy
        // diffs in committed sites.
        let scraps = [
            Scrap::new("s1", &None, "#[[delta]]"),
            Scrap::new("s2", &None, "#[[charlie]]"),
            Scrap::new("s3", &None, "#[[bravo]]"),
            Scrap::new("s4", &None, "#[[alpha]]"),
        ];

        let first = tags_tera(&scraps);
        for _ in 0..100 {
            assert_eq!(titles(&tags_tera(&scraps)), titles(&first));
        }
    }

    #[test]
    fn it_orders_hierarchical_tags_by_full_path() {
        let scraps = [
            Scrap::new("s1", &None, "#[[ai/ml]]"),
            Scrap::new("s2", &None, "#[[ai/agent]]"),
        ];

        assert_eq!(titles(&tags_tera(&scraps)), vec!["ai", "ai/agent", "ai/ml"]);
    }
}
