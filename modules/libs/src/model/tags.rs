use std::collections::HashSet;

use super::{scrap::Scrap, tag::Tag, title::Title};

#[derive(PartialEq, Debug)]
pub struct Tags {
    pub values: HashSet<Tag>,
}

impl Tags {
    pub fn new(scraps: &[Scrap]) -> Tags {
        let scrap_links: HashSet<Title> = scraps
            .iter()
            .flat_map(|scrap| scrap.links.clone())
            .collect();
        let scrap_titles: HashSet<Title> = scraps.iter().map(|scrap| scrap.title.clone()).collect();

        let titles: Vec<Title> = scrap_links
            .into_iter()
            .filter(|link| !scrap_titles.contains(link))
            .collect();

        Tags {
            values: titles.iter().map(|t| t.clone().into()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;

    #[test]
    fn it_new() {
        let base_url = Url::parse("http://localhost:1112/").unwrap();
        let scrap1 = Scrap::new(&base_url, "scrap1", "[[tag1]]");
        let scrap2 = Scrap::new(&base_url, "scrap2", "[[scrap1]]");
        let scraps = vec![scrap1.to_owned(), scrap2.to_owned()];

        let tags = Tags::new(&scraps);
        assert_eq!(
            tags.values.into_iter().collect::<Vec<Tag>>(),
            vec!["tag1".into()]
        )
    }
}
