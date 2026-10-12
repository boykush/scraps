use std::collections::BTreeMap;

use scraps_libs::model::page_path::PagePath;

use crate::usecase::build::model::{
    backlinks_map::BacklinksMap,
    scrap_detail::{ScrapDetail, ScrapDetails},
    title_group::TitleGroup,
};

#[derive(serde::Serialize, PartialEq, Debug)]
struct SerializeTitleIndexScrap {
    ctx: Option<String>,
    title: String,
    /// Page URL relative to `base_url`, already percent-encoded.
    url_path: String,
    backlinks_count: usize,
}

#[derive(serde::Serialize, PartialEq, Debug)]
pub struct TitleIndexGroupTera {
    label: String,
    scraps: Vec<SerializeTitleIndexScrap>,
}

#[derive(serde::Serialize, PartialEq, Debug)]
pub struct TitleIndexTera(Vec<TitleIndexGroupTera>);

impl TitleIndexTera {
    pub fn new(scrap_details: &ScrapDetails, backlinks_map: &BacklinksMap) -> TitleIndexTera {
        Self::of(scrap_details.as_slice().iter(), backlinks_map)
    }

    /// The same index over the scraps of one type only.
    pub fn of_type(
        scrap_details: &ScrapDetails,
        backlinks_map: &BacklinksMap,
        name: &str,
    ) -> TitleIndexTera {
        let typed = scrap_details
            .as_slice()
            .iter()
            .filter(|scrap_detail| scrap_detail.scrap().scrap_type() == Some(name));
        Self::of(typed, backlinks_map)
    }

    fn of<'a>(
        scrap_details: impl Iterator<Item = &'a ScrapDetail>,
        backlinks_map: &BacklinksMap,
    ) -> TitleIndexTera {
        let mut grouped: BTreeMap<TitleGroup, Vec<SerializeTitleIndexScrap>> = BTreeMap::new();
        for scrap_detail in scrap_details {
            let scrap = scrap_detail.scrap();
            let title = scrap.title().to_string();
            let entry = SerializeTitleIndexScrap {
                ctx: scrap.ctx().as_ref().map(|c| c.to_string()),
                url_path: PagePath::of_scrap(&scrap.self_key()).to_url_path(),
                backlinks_count: backlinks_map.get(&scrap.self_key()).len(),
                title,
            };
            grouped
                .entry(TitleGroup::from_title(&entry.title))
                .or_default()
                .push(entry);
        }

        let groups = grouped
            .into_iter()
            .map(|(group, mut scraps)| {
                scraps.sort_by_key(|s| s.title.to_lowercase());
                TitleIndexGroupTera {
                    label: group.label(),
                    scraps,
                }
            })
            .collect();

        TitleIndexTera(groups)
    }
}

#[cfg(test)]
mod tests {
    use scraps_libs::model::{base_url::BaseUrl, scrap::Scrap};
    use url::Url;

    use crate::usecase::build::model::scrap_detail::ScrapDetail;

    use super::*;

    #[test]
    fn it_groups_and_orders_titles() {
        let base_url = &BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let scrap1 = Scrap::new("デザイントークン", &None, "[[DTCG]]");
        let scrap2 = Scrap::new("DTCG", &None, "");
        let scrap3 = Scrap::new("DevOps", &None, "");
        let scrap4 = Scrap::new("設計", &None, "");
        let scraps = [
            scrap1.clone(),
            scrap2.clone(),
            scrap3.clone(),
            scrap4.clone(),
        ];
        let scrap_texts = scraps
            .iter()
            .map(|s| (s.self_key(), s.md_text().to_string()))
            .collect();

        let details = ScrapDetails::new(vec![
            ScrapDetail::new(&scrap1, &None, base_url, &scrap_texts),
            ScrapDetail::new(&scrap2, &None, base_url, &scrap_texts),
            ScrapDetail::new(&scrap3, &None, base_url, &scrap_texts),
            ScrapDetail::new(&scrap4, &None, base_url, &scrap_texts),
        ]);
        let backlinks_map = BacklinksMap::new(&scraps);

        let result = TitleIndexTera::new(&details, &backlinks_map);

        let labels: Vec<&str> = result.0.iter().map(|g| g.label.as_str()).collect();
        assert_eq!(labels, vec!["た", "D", "漢字"]);

        // Within a group, titles sort case-insensitively: DTCG > DevOps.
        let d_titles: Vec<&str> = result.0[1]
            .scraps
            .iter()
            .map(|s| s.title.as_str())
            .collect();
        assert_eq!(d_titles, vec!["DevOps", "DTCG"]);

        // Backlink counts ride along for the index rows.
        assert_eq!(result.0[1].scraps[1].backlinks_count, 1);
    }
}
