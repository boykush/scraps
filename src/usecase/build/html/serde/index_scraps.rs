use itertools::Itertools;
use scraps_libs::model::file::ScrapFileStem;
use url::Url;

use crate::usecase::build::model::{
    backlinks_map::BacklinksMap,
    scrap_detail::{ScrapDetail, ScrapDetails},
    sort::SortKey,
};

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
struct SerializeIndexScrap {
    ctx: Option<String>,
    title: String,
    html_file_name: String,
    html_text: String,
    thumbnail: Option<Url>,
    pub commited_ts: Option<i64>,
    pub backlinks_count: usize,
}

impl SerializeIndexScrap {
    pub fn new(scrap_detail: &ScrapDetail, backlinks_map: &BacklinksMap) -> SerializeIndexScrap {
        let scrap = scrap_detail.scrap();
        let content = scrap_detail.content();
        let commited_ts = scrap_detail.commited_ts();
        let backlinks_count = backlinks_map.get(&scrap.self_link()).len();
        let html_file_name = format!("{}.html", ScrapFileStem::from(scrap.self_link().clone()));
        SerializeIndexScrap {
            ctx: scrap.ctx.map(|c| c.to_string()),
            title: scrap.title.to_string(),
            html_file_name,
            html_text: content.to_string(),
            thumbnail: scrap.thumbnail.clone(),
            commited_ts,
            backlinks_count,
        }
    }
}

#[derive(serde::Serialize, PartialEq, Debug, Clone)]
pub struct IndexScrapsTera(Vec<SerializeIndexScrap>);

impl IndexScrapsTera {
    pub fn new_with_sort(
        scrap_details: &ScrapDetails,
        backlinks_map: &BacklinksMap,
        sort_key: &SortKey,
    ) -> IndexScrapsTera {
        let serialize_scraps = scrap_details
            .to_vec()
            .into_iter()
            .map(|s| SerializeIndexScrap::new(&s, backlinks_map));
        let sorted = (match sort_key {
            SortKey::CommittedDate => serialize_scraps.sorted_by_key(|s| s.commited_ts).rev(),
            SortKey::LinkedCount => serialize_scraps.sorted_by_key(|s| s.backlinks_count).rev(),
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
        let base_url = &Url::parse("http://localhost:1112/").unwrap();
        let sc1 = ScrapDetail::new(
            &Scrap::new("title1", &None, "[[Context/title4]][[title2]]"),
            &None,
            base_url,
        );
        let sc2 = ScrapDetail::new(
            &Scrap::new("title2", &None, "[[Context/title4]][[title1]]"),
            &Some(3),
            base_url,
        );
        let sc3 = ScrapDetail::new(
            &Scrap::new("title3", &None, "[[Context/title4]]"),
            &Some(2),
            base_url,
        );
        let sc4 = ScrapDetail::new(
            &Scrap::new("title4", &Some("Context"), "[[title1]]"),
            &Some(1),
            base_url,
        );
        let backlinks_map =
            BacklinksMap::new(&vec![sc1.scrap(), sc2.scrap(), sc3.scrap(), sc4.scrap()]);

        let sscrap1 = SerializeIndexScrap::new(&sc1.clone(), &backlinks_map);
        let sscrap2 = SerializeIndexScrap::new(&sc2.clone(), &backlinks_map);
        let sscrap3 = SerializeIndexScrap::new(&sc3.clone(), &backlinks_map);
        let sscrap4 = SerializeIndexScrap::new(&sc4.clone(), &backlinks_map);

        // Sort by commited date
        let result1 = IndexScrapsTera::new_with_sort(
            &ScrapDetails::new(&vec![sc1.clone(), sc2.clone(), sc3.clone(), sc4.clone()]),
            &backlinks_map,
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
            &ScrapDetails::new(&vec![sc1.clone(), sc2.clone(), sc3.clone(), sc4.clone()]),
            &backlinks_map,
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
