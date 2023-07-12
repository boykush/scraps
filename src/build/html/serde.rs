use itertools::Itertools;
use url::Url;

use crate::build::model::{scrap::Scrap, linked_scraps_map::LinkedScrapsMap};

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct SerializeScrap {
    title: String,
    links: Vec<String>,
    html_content: String,
    thumbnail: Option<Url>,
    commited_ts: Option<i64>,
    linked_count: usize
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
            linked_count: linked_count
        }
    }
}

#[derive(serde::Serialize, PartialEq, Debug)]
pub struct SerializeScraps(Vec<SerializeScrap>);

impl SerializeScraps {
    pub fn new_with_sort(scraps: &Vec<SerializeScrap>) -> SerializeScraps {
        let sorted = scraps
            .iter()
            .sorted_by_key(|s| s.commited_ts)
            .rev()
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
        let linked_scraps_map = LinkedScrapsMap::new(&vec![]);

        let scrap1 = SerializeScrap::new(&Scrap::new("title1", "text1", &Some(1)), &linked_scraps_map);
        let scrap2 = SerializeScrap::new(&Scrap::new("title2", "text2", &Some(0)), &linked_scraps_map);
        let scrap3 = SerializeScrap::new(&Scrap::new("title3", "text3", &None), &linked_scraps_map);
        let scrap4 = SerializeScrap::new(&Scrap::new("title4", "text4", &Some(2)), &linked_scraps_map);

        let scraps = SerializeScraps::new_with_sort(&vec![
            scrap1.clone(),
            scrap2.clone(),
            scrap3.clone(),
            scrap4.clone(),
        ]);

        assert_eq!(
            scraps.0,
            vec![
                scrap4.clone(),
                scrap1.clone(),
                scrap2.clone(),
                scrap3.clone()
            ]
        )
    }
}
