use itertools::Itertools;
use scraps_libs::model::file::ScrapFileStem;
use url::Url;

use crate::usecase::build::model::{
    backlinks_map::BacklinksMap,
    scrap_with_commited_ts::{ScrapWithCommitedTs, ScrapsWithCommitedTs},
    sort::SortKey,
};

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
struct SerializeIndexScrap {
    ctx: Option<String>,
    title: String,
    html_file_name: String,
    html_content: String,
    thumbnail: Option<Url>,
    pub commited_ts: Option<i64>,
    pub linked_count: usize,
}

impl SerializeIndexScrap {
    pub fn new(
        scrap_with_commited_ts: &ScrapWithCommitedTs,
        linked_scraps_map: &BacklinksMap,
    ) -> SerializeIndexScrap {
        let scrap = scrap_with_commited_ts.scrap();
        let commited_ts = scrap_with_commited_ts.commited_ts();
        let linked_count = linked_scraps_map.get(&scrap.self_link()).len();
        let html_file_name = format!("{}.html", ScrapFileStem::from(scrap.self_link().clone()));
        SerializeIndexScrap {
            ctx: scrap.ctx.map(|c| c.to_string()),
            title: scrap.title.to_string(),
            html_file_name,
            html_content: scrap.html_content.clone(),
            thumbnail: scrap.thumbnail.clone(),
            commited_ts,
            linked_count,
        }
    }
}

#[derive(serde::Serialize, PartialEq, Debug)]
pub struct IndexScrapsTera(Vec<SerializeIndexScrap>);

impl IndexScrapsTera {
    pub fn new_with_sort(
        scraps_with_commited_ts: &ScrapsWithCommitedTs,
        linked_scraps_map: &BacklinksMap,
        sort_key: &SortKey,
    ) -> IndexScrapsTera {
        let serialize_scraps = scraps_with_commited_ts
            .to_vec()
            .into_iter()
            .map(|s| SerializeIndexScrap::new(&s, linked_scraps_map));
        let sorted = (match sort_key {
            SortKey::CommittedDate => serialize_scraps.sorted_by_key(|s| s.commited_ts).rev(),
            SortKey::LinkedCount => serialize_scraps.sorted_by_key(|s| s.linked_count).rev(),
        })
        .collect_vec();

        IndexScrapsTera(sorted)
    }

    pub fn chunks(&self, chunk_size: usize) -> Vec<IndexScrapsTera> {
        self.0
            .chunks(chunk_size)
            .map(|scraps| IndexScrapsTera(scraps.to_vec()))
            .collect_vec()
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use scraps_libs::model::scrap::Scrap;

    use super::*;

    #[test]
    fn it_new_with_sort() {
        let base_url = Url::parse("http://localhost:1112/").unwrap();
        let sc1 = ScrapWithCommitedTs::new(
            &Scrap::new(&base_url, "title1", &None, "[[Context/title4]][[title2]]"),
            &None,
        );
        let sc2 = ScrapWithCommitedTs::new(
            &Scrap::new(&base_url, "title2", &None, "[[Context/title4]][[title1]]"),
            &Some(3),
        );
        let sc3 = ScrapWithCommitedTs::new(
            &Scrap::new(&base_url, "title3", &None, "[[Context/title4]]"),
            &Some(2),
        );
        let sc4 = ScrapWithCommitedTs::new(
            &Scrap::new(&base_url, "title4", &Some("Context"), "[[title1]]"),
            &Some(1),
        );
        let linked_scraps_map =
            BacklinksMap::new(&vec![sc1.scrap(), sc2.scrap(), sc3.scrap(), sc4.scrap()]);

        let sscrap1 = SerializeIndexScrap::new(&sc1.clone(), &linked_scraps_map);
        let sscrap2 = SerializeIndexScrap::new(&sc2.clone(), &linked_scraps_map);
        let sscrap3 = SerializeIndexScrap::new(&sc3.clone(), &linked_scraps_map);
        let sscrap4 = SerializeIndexScrap::new(&sc4.clone(), &linked_scraps_map);

        // Sort by commited date
        let result1 = IndexScrapsTera::new_with_sort(
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
        let result2 = IndexScrapsTera::new_with_sort(
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
