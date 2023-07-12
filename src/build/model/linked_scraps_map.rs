use std::collections::HashMap;

use crate::build::model::scrap::Scrap;

pub struct LinkedScrapsMap {
    values: HashMap<String, Vec<Scrap>>,
}

impl LinkedScrapsMap {
    pub fn new(scraps: &Vec<Scrap>) -> LinkedScrapsMap {
        let linked_map = Self::gen_linked_map(scraps);
        LinkedScrapsMap { values: linked_map }
    }

    pub fn linked_by(&self, title: &str) -> Vec<Scrap> {
        self.values
            .get(title)
            .map_or_else(|| vec![], |s| s.to_owned())
    }

    fn gen_linked_map(scraps: &Vec<Scrap>) -> HashMap<String, Vec<Scrap>> {
        scraps
            .iter()
            .fold(HashMap::new(), |acc1, scrap| {
                scrap.to_owned().links.iter().fold(acc1, |mut acc2, link| {
                    acc2.entry(link.to_string())
                        .or_insert_with(Vec::new)
                        .push(scrap.to_owned());
                    acc2
                })
            })
            .into_iter()
            .collect::<HashMap<String, Vec<Scrap>>>()
    }
}
