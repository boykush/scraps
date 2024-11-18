use itertools::Itertools;
use url::Url;

use crate::build::model::{
    linked_scraps_map::LinkedScrapsMap,
    scrap_with_commited_ts::{ScrapWithCommitedTs, ScrapsWithCommitedTs},
    sort::SortKey,
};

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct SerializeIndexScrap {
    title: String,
    slug: String,
    html_content: String,
    thumbnail: Option<Url>,
    pub commited_ts: Option<i64>,
    pub linked_count: usize,
}

impl SerializeIndexScrap {
    pub fn new(
        scrap_with_commited_ts: &ScrapWithCommitedTs,
        linked_scraps_map: &LinkedScrapsMap,
    ) -> SerializeIndexScrap {
        let scrap = scrap_with_commited_ts.scrap();
        let commited_ts = scrap_with_commited_ts.commited_ts();
        let linked_count = linked_scraps_map.linked_by(&scrap.title).len();
        SerializeIndexScrap {
            title: scrap.title.to_string(),
            slug: scrap.title.slug.to_string(),
            html_content: scrap.html_content.clone(),
            thumbnail: scrap.thumbnail.clone(),
            commited_ts,
            linked_count,
        }
    }
}

#[derive(serde::Serialize, PartialEq, Debug)]
pub struct SerializeIndexScraps(Vec<SerializeIndexScrap>);

impl SerializeIndexScraps {
    pub fn new_with_sort(
        scraps_with_commited_ts: &ScrapsWithCommitedTs,
        linked_scraps_map: &LinkedScrapsMap,
        sort_key: &SortKey,
    ) -> SerializeIndexScraps {
        let serialize_scraps = scraps_with_commited_ts
            .to_vec()
            .into_iter()
            .map(|s| SerializeIndexScrap::new(&s, linked_scraps_map));
        let sorted = (match sort_key {
            SortKey::CommittedDate => serialize_scraps.sorted_by_key(|s| s.commited_ts).rev(),
            SortKey::LinkedCount => serialize_scraps.sorted_by_key(|s| s.linked_count).rev(),
        })
        .collect_vec();

        SerializeIndexScraps(sorted)
    }

    pub fn chunks(&self, chunk_size: usize) -> Vec<SerializeIndexScraps> {
        self.0
            .chunks(chunk_size)
            .map(|scraps| SerializeIndexScraps(scraps.to_vec()))
            .collect_vec()
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use crate::libs::model::scrap::Scrap;

    use super::*;

    #[test]
    fn it_new_with_sort() {
        let base_url = Url::parse("http://localhost:1112/").unwrap();
        let sc1 = ScrapWithCommitedTs::new(
            &Scrap::new(&base_url, "title1", "[[title4]][[title2]]"),
            &None,
        );
        let sc2 = ScrapWithCommitedTs::new(
            &Scrap::new(&base_url, "title2", "[[title4]][[title1]]"),
            &Some(3),
        );
        let sc3 =
            ScrapWithCommitedTs::new(&Scrap::new(&base_url, "title3", "[[title4]]"), &Some(2));
        let sc4 =
            ScrapWithCommitedTs::new(&Scrap::new(&base_url, "title4", "[[title1]]"), &Some(1));
        let linked_scraps_map =
            LinkedScrapsMap::new(&vec![sc1.scrap(), sc2.scrap(), sc3.scrap(), sc4.scrap()]);

        let sscrap1 = SerializeIndexScrap::new(&sc1.clone(), &linked_scraps_map);
        let sscrap2 = SerializeIndexScrap::new(&sc2.clone(), &linked_scraps_map);
        let sscrap3 = SerializeIndexScrap::new(&sc3.clone(), &linked_scraps_map);
        let sscrap4 = SerializeIndexScrap::new(&sc4.clone(), &linked_scraps_map);

        // Sort by commited date
        let result1 = SerializeIndexScraps::new_with_sort(
            &ScrapsWithCommitedTs::new(&vec![sc1.clone(), sc2.clone(), sc3.clone(), sc4.clone()]),
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
        let result2 = SerializeIndexScraps::new_with_sort(
            &ScrapsWithCommitedTs::new(&vec![sc1.clone(), sc2.clone(), sc3.clone(), sc4.clone()]),
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
