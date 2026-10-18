use std::collections::BTreeMap;

use itertools::Itertools;
use rayon::prelude::*;
use scraps_libs::model::page_path::PagePath;
use url::Url;

use crate::usecase::build::model::{
    backlinks_map::BacklinksMap,
    scrap_detail::{ScrapDetail, ScrapDetails},
    summary::Summary,
    title_group::TitleGroup,
};

/// Long enough to be a useful gloss on an index row, short enough to stay on
/// one line at the narrowest supported width.
const SUMMARY_MAX_CHARS: usize = 90;

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
struct SerializeIndexScrap {
    ctx: Option<String>,
    /// What a wiki-link to the scrap holds: its path without `.md`.
    key: String,
    title: String,
    /// Page URL relative to `base_url`, already percent-encoded.
    url_path: String,
    thumbnail: Option<Url>,
    summary: Option<String>,
    pub commited_ts: Option<i64>,
    pub backlinks_count: usize,
    pub links_count: usize,
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
        let url_path = PagePath::of_scrap(&scrap.self_key()).to_url_path();
        SerializeIndexScrap {
            ctx: scrap.ctx().as_ref().map(|c| c.to_string()),
            key: scrap.self_key().to_string(),
            title: scrap.title().to_string(),
            url_path,
            thumbnail: scrap.thumbnail(),
            summary,
            commited_ts,
            backlinks_count,
            links_count,
            scrap_type: scrap.scrap_type().map(str::to_string),
        }
    }
}

/// Every scrap's row, built once per build in updated order; each scope's
/// list is cut from it, so a row's summary is derived only once.
pub struct IndexScraps(Vec<SerializeIndexScrap>);

impl IndexScraps {
    pub fn new(scrap_details: &ScrapDetails, backlinks_map: &BacklinksMap) -> IndexScraps {
        let rows = scrap_details
            .as_slice()
            .par_iter()
            .map(|s| SerializeIndexScrap::new(s, backlinks_map))
            .collect::<Vec<_>>();
        IndexScraps(
            rows.into_iter()
                .sorted_by_key(|s| s.commited_ts)
                .rev()
                .collect_vec(),
        )
    }

    /// The list of one scope: the rows whose key `keep` accepts, with the
    /// rank each takes under every sort, so the page can reorder itself.
    pub fn scope(&self, keep: impl Fn(&str) -> bool) -> IndexScrapsTera {
        let scraps = self.0.iter().filter(|s| keep(&s.key)).collect_vec();

        let mut backlinks_rank = vec![0; scraps.len()];
        let by_backlinks = (0..scraps.len())
            .sorted_by(|&a, &b| scraps[b].backlinks_count.cmp(&scraps[a].backlinks_count));
        for (rank, index) in by_backlinks.enumerate() {
            backlinks_rank[index] = rank;
        }

        // Title order interleaves each group's heading before its rows.
        let mut grouped: BTreeMap<TitleGroup, Vec<usize>> = BTreeMap::new();
        for (index, scrap) in scraps.iter().enumerate() {
            grouped
                .entry(TitleGroup::from_title(&scrap.title))
                .or_default()
                .push(index);
        }
        let mut title_rank = vec![(0, 0); scraps.len()];
        let mut groups = Vec::with_capacity(grouped.len());
        let mut rank = 0;
        for (number, (group, mut members)) in grouped.into_iter().enumerate() {
            let number = number + 1;
            groups.push(TitleGroupTera {
                label: group.label(),
                number,
                title_rank: rank,
            });
            rank += 1;
            members.sort_by_key(|&index| scraps[index].title.to_lowercase());
            for index in members {
                title_rank[index] = (rank, number);
                rank += 1;
            }
        }

        let rows = scraps
            .into_iter()
            .enumerate()
            .map(|(index, scrap)| IndexRowTera {
                scrap: scrap.clone(),
                updated_rank: index,
                backlinks_rank: backlinks_rank[index],
                title_rank: title_rank[index].0,
                group: title_rank[index].1,
            })
            .collect();
        IndexScrapsTera { rows, groups }
    }
}

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
struct IndexRowTera {
    #[serde(flatten)]
    scrap: SerializeIndexScrap,
    updated_rank: usize,
    backlinks_rank: usize,
    title_rank: usize,
    /// The number of the title group the row falls in, from 1.
    group: usize,
}

/// A heading of the title order: the title's initial, as the all-scraps
/// index grouped it.
#[derive(serde::Serialize, Clone, PartialEq, Debug)]
struct TitleGroupTera {
    label: String,
    number: usize,
    title_rank: usize,
}

#[derive(serde::Serialize, PartialEq, Debug, Clone)]
pub struct IndexScrapsTera {
    rows: Vec<IndexRowTera>,
    groups: Vec<TitleGroupTera>,
}

#[cfg(test)]
mod tests {
    use url::Url;

    use scraps_libs::model::{base_url::BaseUrl, scrap::Scrap};

    use super::*;

    fn summary_of(md_text: &str) -> Option<String> {
        let base_url = &BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let scrap = Scrap::new("orders", &None, md_text);
        let link_targets = scraps_libs::html::LinkTargets::new([&scrap]);
        let detail = ScrapDetail::new(&scrap, &None, base_url, &link_targets);
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

    fn index_scraps(scraps: &[Scrap], ts: &[Option<i64>]) -> IndexScrapsTera {
        let base_url = &BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let link_targets = scraps_libs::html::LinkTargets::new(scraps);
        let details = ScrapDetails::new(
            scraps
                .iter()
                .zip(ts)
                .map(|(scrap, ts)| ScrapDetail::new(scrap, ts, base_url, &link_targets))
                .collect(),
        );
        IndexScraps::new(&details, &BacklinksMap::new(scraps)).scope(|_| true)
    }

    fn titles_by(list: &IndexScrapsTera, rank: impl Fn(&IndexRowTera) -> usize) -> Vec<&str> {
        list.rows
            .iter()
            .sorted_by_key(|row| rank(row))
            .map(|row| row.scrap.title.as_str())
            .collect()
    }

    #[test]
    fn it_ranks_every_row_under_each_sort() {
        let scraps = [
            Scrap::new("title1", &None, "[[Context/title4]][[title2]]"),
            Scrap::new("title2", &None, "[[Context/title4]][[title1]]"),
            Scrap::new("title3", &None, "[[Context/title4]]"),
            Scrap::new("title4", &Some("Context".into()), "[[title1]]"),
        ];
        let list = index_scraps(&scraps, &[None, Some(3), Some(2), Some(1)]);

        // Rows come in updated order, which is also the page's order without JS.
        let in_order: Vec<&str> = list.rows.iter().map(|r| r.scrap.title.as_str()).collect();
        assert_eq!(in_order, vec!["title2", "title3", "title4", "title1"]);
        assert_eq!(
            titles_by(&list, |r| r.updated_rank),
            vec!["title2", "title3", "title4", "title1"]
        );
        // Most linked first; a tie keeps the updated order.
        assert_eq!(
            titles_by(&list, |r| r.backlinks_rank),
            vec!["title4", "title1", "title2", "title3"]
        );
    }

    #[test]
    fn it_groups_the_title_order_by_initial() {
        let scraps = [
            Scrap::new("デザイントークン", &None, "[[DTCG]]"),
            Scrap::new("DTCG", &None, ""),
            Scrap::new("DevOps", &None, ""),
            Scrap::new("設計", &None, ""),
            Scrap::new("orders", &None, "---\ntitle: Customer Orders\n---\n"),
        ];
        let list = index_scraps(&scraps, &[None, None, None, None, None]);

        let labels: Vec<&str> = list.groups.iter().map(|g| g.label.as_str()).collect();
        assert_eq!(labels, vec!["た", "C", "D", "漢字"]);
        // Within a group, the shown title sorts case-insensitively.
        assert_eq!(
            titles_by(&list, |r| r.title_rank),
            vec![
                "デザイントークン",
                "Customer Orders",
                "DevOps",
                "DTCG",
                "設計"
            ]
        );
        // Each heading ranks just before its group's first row.
        let d = &list.groups[2];
        let first_d = list
            .rows
            .iter()
            .find(|r| r.scrap.title == "DevOps")
            .unwrap();
        assert_eq!(first_d.group, d.number);
        assert_eq!(first_d.title_rank, d.title_rank + 1);
    }

    #[test]
    fn it_cuts_a_scope_out_of_every_row() {
        let base_url = &BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let scraps = [Scrap::new("a", &None, ""), Scrap::new("b", &None, "")];
        let link_targets = scraps_libs::html::LinkTargets::new(&scraps);
        let details = ScrapDetails::new(
            scraps
                .iter()
                .map(|scrap| ScrapDetail::new(scrap, &None, base_url, &link_targets))
                .collect(),
        );
        let list = IndexScraps::new(&details, &BacklinksMap::new(&scraps)).scope(|key| key == "b");
        assert_eq!(list.rows.len(), 1);
        assert_eq!(list.rows[0].updated_rank, 0);
    }
}
