use std::collections::HashMap;

use crate::libs::model::{scrap::Scrap, title::Title};

pub struct LinkedScrapsMap {
    values: HashMap<Title, Vec<Scrap>>,
}

impl LinkedScrapsMap {
    pub fn new(scraps: &[Scrap]) -> LinkedScrapsMap {
        let linked_map = Self::gen_linked_map(scraps);
        LinkedScrapsMap { values: linked_map }
    }

    pub fn linked_by(&self, title: &Title) -> Vec<Scrap> {
        self.values.get(title).map_or_else(Vec::new, Vec::clone)
    }

    fn gen_linked_map(scraps: &[Scrap]) -> HashMap<Title, Vec<Scrap>> {
        scraps
            .iter()
            .fold(HashMap::new(), |acc1: HashMap<Title, Vec<Scrap>>, scrap| {
                scrap.to_owned().links.iter().fold(acc1, |mut acc2, link| {
                    acc2.entry(link.to_owned())
                        .or_default()
                        .push(scrap.to_owned());
                    acc2
                })
            })
            .into_iter()
            .collect::<HashMap<Title, Vec<Scrap>>>()
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;

    #[test]
    fn it_linked_by() {
        let base_url = Url::parse("http://localhost:1112/").unwrap();
        let scrap1 = Scrap::new(&base_url, "scrap1", "[[tag1]]");
        let scrap2 = Scrap::new(&base_url, "scrap2", "[[scrap1]][[tag1]]");
        let scraps = vec![scrap1.to_owned(), scrap2.to_owned()];

        let linked_map = LinkedScrapsMap::new(&scraps);
        // scraps links
        assert_eq!(
            linked_map.linked_by(&Title::new("scrap1")),
            vec![scrap2.to_owned()]
        );
        // tags
        assert_eq!(
            linked_map.linked_by(&Title::new("tag1")),
            vec![scrap1.to_owned(), scrap2.to_owned()]
        )
    }
}
