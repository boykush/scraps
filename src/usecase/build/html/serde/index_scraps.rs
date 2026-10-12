use itertools::Itertools;
use rayon::prelude::*;
use scraps_libs::model::file::ScrapFileStem;
use url::Url;

use crate::usecase::build::model::{
    backlinks_map::BacklinksMap,
    scrap_detail::{ScrapDetail, ScrapDetails},
    sort::SortKey,
    summary::Summary,
};

/// Long enough to be a useful gloss on an index row, short enough to stay on
/// one line at the narrowest supported width.
const SUMMARY_MAX_CHARS: usize = 90;

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
struct SerializeIndexScrap {
    ctx: Option<String>,
    title: String,
    html_file_name: String,
    thumbnail: Option<Url>,
    summary: Option<String>,
    pub commited_ts: Option<i64>,
    pub backlinks_count: usize,
    pub links_count: usize,
    #[serde(skip)]
    scrap_type: Option<String>,
}

impl SerializeIndexScrap {
    pub fn new(scrap_detail: &ScrapDetail, backlinks_map: &BacklinksMap) -> SerializeIndexScrap {
        let scrap = scrap_detail.scrap();
        let commited_ts = scrap_detail.commited_ts();
        let backlinks_count = backlinks_map.get(&scrap.self_key()).len();
        let links_count = scrap.links().len();
        // The author's own sentence is shown as written; only a summary
        // derived from the body needs cutting down to a row.
        let summary = scrap.description().map(str::to_string).or_else(|| {
            Summary::from_md_text(scrap.md_text(), SUMMARY_MAX_CHARS)
                .map(|s| s.as_str().to_string())
        });
        let html_file_name = format!("{}.html", ScrapFileStem::from(scrap.self_key().clone()));
        SerializeIndexScrap {
            ctx: scrap.ctx().as_ref().map(|c| c.to_string()),
            title: scrap.title().to_string(),
            html_file_name,
            thumbnail: scrap.thumbnail(),
            summary,
            commited_ts,
            backlinks_count,
            links_count,
            scrap_type: scrap.scrap_type().map(str::to_string),
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
            .as_slice()
            .par_iter()
            .map(|s| SerializeIndexScrap::new(s, backlinks_map))
            .collect::<Vec<_>>()
            .into_iter();
        let sorted = (match sort_key {
            SortKey::CommittedDate => serialize_scraps.sorted_by_key(|s| s.commited_ts).rev(),
            SortKey::LinkedCount => serialize_scraps.sorted_by_key(|s| s.backlinks_count).rev(),
        })
        .collect_vec();

        IndexScrapsTera(sorted)
    }

    /// The rows of one type, in the order this view already sorted them.
    pub fn of_type(&self, name: &str) -> IndexScrapsTera {
        let typed = self
            .0
            .iter()
            .filter(|scrap| scrap.scrap_type.as_deref() == Some(name));
        IndexScrapsTera(typed.cloned().collect_vec())
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

    use scraps_libs::model::{base_url::BaseUrl, scrap::Scrap};

    use super::*;

    fn summary_of(md_text: &str) -> Option<String> {
        let base_url = &BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let scrap = Scrap::new("orders", &None, md_text);
        let scrap_texts = [(scrap.self_key(), scrap.md_text().to_string())]
            .into_iter()
            .collect();
        let detail = ScrapDetail::new(&scrap, &None, base_url, &scrap_texts);
        let backlinks_map = BacklinksMap::new(std::slice::from_ref(&scrap));

        SerializeIndexScrap::new(&detail, &backlinks_map).summary
    }

    #[test]
    fn it_summarises_with_the_description_when_the_scrap_has_one() {
        let described =
            "---\ndescription: One row per completed customer order.\n---\n\n# Orders table\n";
        assert_eq!(
            summary_of(described).as_deref(),
            Some("One row per completed customer order.")
        );
    }

    #[rstest::rstest]
    #[case::no_frontmatter("# Orders table\n")]
    #[case::no_description("---\ntype: Table\n---\n\n# Orders table\n")]
    #[case::blank_description("---\ndescription: ''\n---\n\n# Orders table\n")]
    fn it_summarises_from_the_body_without_a_description(#[case] md_text: &str) {
        assert_eq!(summary_of(md_text).as_deref(), Some("Orders table"));
    }

    #[test]
    fn it_new_with_sort() {
        let base_url = &BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let scrap1 = Scrap::new("title1", &None, "[[Context/title4]][[title2]]");
        let scrap2 = Scrap::new("title2", &None, "[[Context/title4]][[title1]]");
        let scrap3 = Scrap::new("title3", &None, "[[Context/title4]]");
        let scrap4 = Scrap::new("title4", &Some("Context".into()), "[[title1]]");
        let scraps = [
            scrap1.clone(),
            scrap2.clone(),
            scrap3.clone(),
            scrap4.clone(),
        ];
        let scrap_texts = scraps
            .iter()
            .map(|scrap| (scrap.self_key(), scrap.md_text().to_string()))
            .collect();

        let sc1 = ScrapDetail::new(&scrap1, &None, base_url, &scrap_texts);
        let sc2 = ScrapDetail::new(&scrap2, &Some(3), base_url, &scrap_texts);
        let sc3 = ScrapDetail::new(&scrap3, &Some(2), base_url, &scrap_texts);
        let sc4 = ScrapDetail::new(&scrap4, &Some(1), base_url, &scrap_texts);
        let backlinks_map = BacklinksMap::new(&scraps);

        let sscrap1 = SerializeIndexScrap::new(&sc1.clone(), &backlinks_map);
        let sscrap2 = SerializeIndexScrap::new(&sc2.clone(), &backlinks_map);
        let sscrap3 = SerializeIndexScrap::new(&sc3.clone(), &backlinks_map);
        let sscrap4 = SerializeIndexScrap::new(&sc4.clone(), &backlinks_map);

        // Sort by commited date
        let result1 = IndexScrapsTera::new_with_sort(
            &ScrapDetails::new(vec![sc1.clone(), sc2.clone(), sc3.clone(), sc4.clone()]),
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
            &ScrapDetails::new(vec![sc1.clone(), sc2.clone(), sc3.clone(), sc4.clone()]),
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
