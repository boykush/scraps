use itertools::Itertools;
use url::Url;

use crate::build::model::{linked_scraps_map::LinkedScrapsMap, scrap::Scrap, sort::SortKey};

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct SerializeScrap {
    title: String,
    links: Vec<String>,
    html_content: String,
    thumbnail: Option<Url>,
    commited_ts: Option<i64>,
    linked_count: usize,
}

impl SerializeScrap {
    pub fn new(scrap: &Scrap, linked_scraps_map: &LinkedScrapsMap) -> SerializeScrap {
        let linked_count = linked_scraps_map.linked_by(&scrap.title).len();
        SerializeScrap {
            title: scrap.title.to_owned(),
            links: scrap.links.to_owned(),
            html_content: scrap.html_content.to_owned(),
            thumbnail: scrap.thumbnail.to_owned(),
            commited_ts: scrap.commited_ts.to_owned(),
            linked_count: linked_count,
        }
    }
}

#[derive(serde::Serialize, PartialEq, Debug)]
pub struct SerializeScraps(Vec<SerializeScrap>);

impl SerializeScraps {
    pub fn new_with_sort(scraps: &Vec<SerializeScrap>, sort_key: &SortKey) -> SerializeScraps {
        let sorted = (match sort_key {
            SortKey::CommitedDate => scraps.iter().sorted_by_key(|s| s.commited_ts).rev(),
            SortKey::LinkedCount => scraps.iter().sorted_by_key(|s| s.linked_count).rev(),
        })
        .cloned()
        .collect_vec();

        SerializeScraps(sorted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_new_with_sort() {
        let scrap1 = &Scrap::new("title1", "[[title4]][[title2]]", &None);
        let scrap2 = &Scrap::new("title2", "[[title4]][[title1]]", &Some(3));
        let scrap3 = &Scrap::new("title3", "[[title4]]", &Some(2));
        let scrap4 = &Scrap::new("title4", "[[title1]]", &Some(1));
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
                sscrap1.clone(),
                sscrap2.clone(),
                sscrap3.clone(),
                sscrap4.clone(),
            ],
            &SortKey::CommitedDate,
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
                sscrap1.clone(),
                sscrap2.clone(),
                sscrap3.clone(),
                sscrap4.clone(),
            ],
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
