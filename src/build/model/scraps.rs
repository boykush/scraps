use std::collections::HashMap;

use crate::build::model::scrap::Scrap;

pub struct Scraps(Vec<Scrap>);

impl Scraps {
    pub fn new(scraps: &Vec<Scrap>) -> Scraps {
        Scraps(scraps.to_owned())
    }

    pub fn gen_linked_map(&self) -> HashMap<String, Vec<Scrap>> {
        self.0
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
