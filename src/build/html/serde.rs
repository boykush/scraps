use itertools::Itertools;
use url::Url;

use crate::build::model::scrap::Scrap;

#[derive(serde::Serialize)]
#[serde(remote = "Scrap")]
struct SScrap {
    title: String,
    links: Vec<String>,
    html_content: String,
    thumbnail: Option<Url>,
    updated_ts: u64,
}

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct SerializeScrap(#[serde(with = "SScrap")] Scrap);

impl SerializeScrap {
    pub fn new(scrap: &Scrap) -> SerializeScrap {
        SerializeScrap(scrap.to_owned())
    }
}

#[derive(serde::Serialize, PartialEq, Debug)]
pub struct SerializeScraps(Vec<SerializeScrap>);

impl SerializeScraps {
    pub fn new_with_sort(scraps: &Vec<SerializeScrap>) -> SerializeScraps {
        let sorted = scraps
            .iter()
            .sorted_by_key(|s| s.0.updated_ts)
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
        let scrap1 = SerializeScrap::new(&Scrap::new("title1", "text1", &1));
        let scrap2 = SerializeScrap::new(&Scrap::new("title2", "text2", &0));
        let scrap3 = SerializeScrap::new(&Scrap::new("title3", "text3", &2));

        let scraps =
            SerializeScraps::new_with_sort(&vec![scrap1.clone(), scrap2.clone(), scrap3.clone()]);

        assert_eq!(
            scraps.0,
            vec![scrap3.clone(), scrap1.clone(), scrap2.clone(),]
        )
    }
}
