use std::collections::HashSet;

use super::{key::ScrapKey, scrap::Scrap, tag::Tag, title::Title};

#[derive(PartialEq, Debug, Clone)]
pub struct Tags(HashSet<Tag>);

impl IntoIterator for Tags {
    type Item = Tag;
    type IntoIter = std::collections::hash_set::IntoIter<Tag>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Tags {
    pub fn new(scraps: &[Scrap]) -> Tags {
        let scrap_links: HashSet<ScrapKey> = scraps
            .iter()
            .flat_map(|scrap| scrap.links().to_vec())
            .collect();
        let scrap_self_keys: HashSet<ScrapKey> =
            scraps.iter().map(|scrap| scrap.self_key()).collect();

        let links: Vec<ScrapKey> = scrap_links
            .into_iter()
            .filter(|key| !scrap_self_keys.contains(key))
            .collect();

        Tags(links.iter().map(|l| Title::from(l).into()).collect())
    }

    pub fn iter(&self) -> std::collections::hash_set::Iter<'_, Tag> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_new() {
        let scrap1 = Scrap::new("scrap1", &None, "[[tag1]]");
        let scrap2 = Scrap::new("scrap2", &None, "[[scrap1]]");
        let scraps = vec![scrap1.to_owned(), scrap2.to_owned()];

        let tags = Tags::new(&scraps);
        assert_eq!(tags.into_iter().collect::<Vec<Tag>>(), vec!["tag1".into()])
    }
}
