use itertools::Itertools;

use crate::{
    build::model::{linked_scraps_map::LinkedScrapsMap, sort::SortKey},
    libs::model::scrap::Scrap,
};

use super::scrap::SerializeScrap;

#[derive(serde::Serialize, PartialEq, Debug)]
pub struct SerializeScraps(Vec<SerializeScrap>);

impl SerializeScraps {
    pub fn new_with_sort(
        scraps: &[Scrap],
        linked_scraps_map: &LinkedScrapsMap,
        sort_key: &SortKey,
    ) -> SerializeScraps {
        let serialize_scraps = scraps
            .iter()
            .map(|s| SerializeScrap::new(s, linked_scraps_map));
        let sorted = (match sort_key {
            SortKey::CommittedDate => serialize_scraps.sorted_by_key(|s| s.commited_ts).rev(),
            SortKey::LinkedCount => serialize_scraps.sorted_by_key(|s| s.linked_count).rev(),
        })
        .collect_vec();

        SerializeScraps(sorted)
    }

    pub fn chunks(&self, chunk_size: usize) -> Vec<SerializeScraps> {
        self.0
            .chunks(chunk_size)
            .map(|scraps| SerializeScraps(scraps.to_vec()))
            .collect_vec()
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;

    #[test]
    fn it_new_with_sort() {
        let base_url = Url::parse("http://localhost:1112/").unwrap();
        let scrap1 = &Scrap::new(&base_url, "title1", "[[title4]][[title2]]", &None);
        let scrap2 = &Scrap::new(&base_url, "title2", "[[title4]][[title1]]", &Some(3));
        let scrap3 = &Scrap::new(&base_url, "title3", "[[title4]]", &Some(2));
        let scrap4 = &Scrap::new(&base_url, "title4", "[[title1]]", &Some(1));
        let linked_scraps_map = LinkedScrapsMap::new(&vec![
            scrap1.clone(),
            scrap2.clone(),
            scrap3.clone(),
            scrap4.clone(),
        ]);

        let sscrap1 = SerializeScrap::new(&scrap1.clone(), &linked_scraps_map);
        let sscrap2 = SerializeScrap::new(&scrap2.clone(), &linked_scraps_map);
        let sscrap3 = SerializeScrap::new(&scrap3.clone(), &linked_scraps_map);
        let sscrap4 = SerializeScrap::new(&scrap4.clone(), &linked_scraps_map);

        // Sort by commited date
        let result1 = SerializeScraps::new_with_sort(
            &vec![
                scrap1.clone(),
                scrap2.clone(),
                scrap3.clone(),
                scrap4.clone(),
            ],
            &linked_scraps_map,
            &SortKey::CommittedDate,
        );

        assert_eq!(
            result1.0,
            vec![
                sscrap2.clone(),
                sscrap3.clone(),
                sscrap4.clone(),
                sscrap1.clone()
            ]
        );

        // Sort by linked count
        let result2 = SerializeScraps::new_with_sort(
            &vec![
                scrap1.clone(),
                scrap2.clone(),
                scrap3.clone(),
                scrap4.clone(),
            ],
            &linked_scraps_map,
            &SortKey::LinkedCount,
        );

        assert_eq!(
            result2.0,
            vec![
                sscrap4.clone(),
                sscrap1.clone(),
                sscrap2.clone(),
                sscrap3.clone()
            ]
        )
    }
}
