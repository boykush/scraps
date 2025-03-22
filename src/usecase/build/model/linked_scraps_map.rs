use std::collections::HashMap;

use scraps_libs::model::{link::ScrapLink, scrap::Scrap};

#[derive(PartialEq, Debug)]
pub struct LinkedScrapsMap(HashMap<ScrapLink, Vec<Scrap>>);

impl LinkedScrapsMap {
    pub fn new(scraps: &[Scrap]) -> LinkedScrapsMap {
        let linked_map = Self::gen_linked_map(scraps);
        LinkedScrapsMap(linked_map)
    }

    pub fn linked_by(&self, link: &ScrapLink) -> Vec<Scrap> {
        self.0.get(link).map_or_else(Vec::new, Vec::clone)
    }

    fn gen_linked_map(scraps: &[Scrap]) -> HashMap<ScrapLink, Vec<Scrap>> {
        scraps
            .iter()
            .fold(
                HashMap::new(),
                |acc1: HashMap<ScrapLink, Vec<Scrap>>, scrap| {
                    scrap.to_owned().links.iter().fold(acc1, |mut acc2, link| {
                        acc2.entry(link.clone()).or_default().push(scrap.to_owned());
                        acc2
                    })
                },
            )
            .into_iter()
            .collect::<HashMap<ScrapLink, Vec<Scrap>>>()
    }
}

#[cfg(test)]
mod tests {
    use scraps_libs::model::title::Title;
    use url::Url;

    use super::*;

    #[test]
    fn it_linked_by() {
        let base_url = Url::parse("http://localhost:1112/").unwrap();
        let scrap1 = Scrap::new(&base_url, "scrap1", &None, "[[tag1]]");
        let scrap2 = Scrap::new(&base_url, "scrap2", &None, "[[scrap1]][[tag1]]");
        let scraps = vec![scrap1.to_owned(), scrap2.to_owned()];

        let linked_map = LinkedScrapsMap::new(&scraps);
        // scraps links
        assert_eq!(
            linked_map.linked_by(&Title::from("scrap1").into()),
            vec![scrap2.to_owned()]
        );
        // tags
        assert_eq!(
            linked_map.linked_by(&Title::from("tag1").into()),
            vec![scrap1.to_owned(), scrap2.to_owned()]
        )
    }

    #[test]
    fn it_linked_by_with_context() {
        let base_url = Url::parse("http://localhost:1112/").unwrap();
        let scrap1 = Scrap::new(&base_url, "scrap1", &Some("Context"), "");
        let scrap2 = Scrap::new(&base_url, "scrap2", &Some("Context"), "[[Context/scrap1]]");
        let scrap3 = Scrap::new(&base_url, "scrap3", &None, "[[Context/scrap1]][[Context/scrap2]]");
        let scraps = vec![scrap1.clone(), scrap2.clone(), scrap3.clone()];

        let linked_map = LinkedScrapsMap::new(&scraps);
        assert_eq!(
            linked_map.linked_by(&ScrapLink::with_ctx(&"scrap1".into(), &"Context".into())),
            vec![scrap2.clone(), scrap3.clone()]
        );
        assert_eq!(
            linked_map.linked_by(&ScrapLink::with_ctx(&"scrap2".into(), &"Context".into())),
            vec![scrap3.clone()]
        );
        assert_eq!(
            linked_map.linked_by(&Title::from("scrap3").into()),
            vec![]
        );
    }
}
