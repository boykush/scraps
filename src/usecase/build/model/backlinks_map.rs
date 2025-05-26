use std::collections::HashMap;

use scraps_libs::model::{link::ScrapLink, scrap::Scrap};

#[derive(PartialEq, Debug)]
pub struct BacklinksMap(HashMap<ScrapLink, Vec<Scrap>>);

impl BacklinksMap {
    pub fn new(scraps: &[Scrap]) -> BacklinksMap {
        let backlinks_map = Self::gen_backlinks_map(scraps);
        BacklinksMap(backlinks_map)
    }

    pub fn get(&self, link: &ScrapLink) -> Vec<Scrap> {
        self.0.get(link).map_or_else(Vec::new, Vec::clone)
    }

    fn gen_backlinks_map(scraps: &[Scrap]) -> HashMap<ScrapLink, Vec<Scrap>> {
        scraps
            .iter()
            .fold(
                HashMap::new(),
                |acc1: HashMap<ScrapLink, Vec<Scrap>>, scrap| {
                    scrap
                        .to_owned()
                        .links()
                        .iter()
                        .fold(acc1, |mut acc2, link| {
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

    use super::*;

    #[test]
    fn it_get() {
        let scrap1 = Scrap::new("scrap1", &None, "[[tag1]]");
        let scrap2 = Scrap::new("scrap2", &None, "[[scrap1]][[tag1]]");
        let scraps = vec![scrap1.to_owned(), scrap2.to_owned()];

        let backlinks_map = BacklinksMap::new(&scraps);
        // scraps links
        assert_eq!(
            backlinks_map.get(&Title::from("scrap1").into()),
            vec![scrap2.to_owned()]
        );
        // tags
        assert_eq!(
            backlinks_map.get(&Title::from("tag1").into()),
            vec![scrap1.to_owned(), scrap2.to_owned()]
        )
    }

    #[test]
    fn it_get_with_context() {
        let scrap1 = Scrap::new("scrap1", &Some("Context"), "");
        let scrap2 = Scrap::new("scrap2", &Some("Context"), "[[Context/scrap1]]");
        let scrap3 = Scrap::new("scrap3", &None, "[[Context/scrap1]][[Context/scrap2]]");
        let scraps = vec![scrap1.clone(), scrap2.clone(), scrap3.clone()];

        let backlinks_map = BacklinksMap::new(&scraps);
        assert_eq!(
            backlinks_map.get(&ScrapLink::with_ctx(&"scrap1".into(), &"Context".into())),
            vec![scrap2.clone(), scrap3.clone()]
        );
        assert_eq!(
            backlinks_map.get(&ScrapLink::with_ctx(&"scrap2".into(), &"Context".into())),
            vec![scrap3.clone()]
        );
        assert_eq!(backlinks_map.get(&Title::from("scrap3").into()), vec![]);
    }
}
