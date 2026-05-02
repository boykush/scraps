use std::collections::HashSet;

use comrak::{
    nodes::{NodeValue, NodeWikiLink},
    options::Extension,
    parse_document, Arena, Options,
};
use url::Url;

use crate::model::key::ScrapKey;

fn options() -> Options<'static> {
    Options {
        extension: Extension {
            wikilinks_title_after_pipe: true,
            ..Extension::default()
        },
        ..Options::default()
    }
}

pub fn head_image(text: &str) -> Option<Url> {
    let arena = Arena::new();
    let opts = options();
    let root = parse_document(&arena, text, &opts);
    for node in root.descendants() {
        if let NodeValue::Image(node_link) = &node.data().value {
            return Url::parse(&node_link.url).ok();
        }
    }
    None
}

pub fn scrap_links(text: &str) -> Vec<ScrapKey> {
    let hashed: HashSet<ScrapKey> = scrap_links_with_duplicates(text).into_iter().collect();
    hashed.into_iter().collect()
}

pub fn scrap_links_with_duplicates(text: &str) -> Vec<ScrapKey> {
    let arena = Arena::new();
    let opts = options();
    let root = parse_document(&arena, text, &opts);
    root.descendants()
        .filter_map(|node| match &node.data().value {
            NodeValue::WikiLink(NodeWikiLink { url }) if !url.is_empty() => {
                Some(ScrapKey::from_path_str(url))
            }
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::model::title::Title;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::image_found(
        "![alt](https://example.com/image.png)",
        Some("https://example.com/image.png")
    )]
    #[case::no_image("# header1", None)]
    fn it_head_image(#[case] input: &str, #[case] expected_url: Option<&str>) {
        assert_eq!(
            head_image(input),
            expected_url.map(|u| Url::parse(u).unwrap())
        );
    }

    #[test]
    fn it_scrap_links_with_duplicates() {
        let text = "[[a]] [[b]] [[a]] [[c]] [[b]] [[a]]";
        let result = scrap_links_with_duplicates(text);
        assert_eq!(result.len(), 6);

        let a_key: ScrapKey = Title::from("a").into();
        let a_count = result.iter().filter(|k| **k == a_key).count();
        assert_eq!(a_count, 3);

        let b_key: ScrapKey = Title::from("b").into();
        let b_count = result.iter().filter(|k| **k == b_key).count();
        assert_eq!(b_count, 2);
    }

    #[test]
    fn it_scrap_links_with_duplicates_no_duplicates() {
        let text = "[[a]] [[b]] [[c]]";
        let result = scrap_links_with_duplicates(text);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn it_scrap_links() {
        let valid_links = &[
            "[[head]]",
            "[[contain space]]",
            "[[last]]",
            "[[duplicate]]",
            "[[duplicate]]",
            "[[Domain Driven Design|DDD]]", // alias by pipe
            "[[Book/Test-driven development]]",
            "[[Person/Eric Evans|Eric Evans]]",
        ]
        .join("\n");
        let mut result1 = scrap_links(valid_links);
        let mut expected1: Vec<ScrapKey> = [
            Title::from("head").into(),
            Title::from("contain space").into(),
            Title::from("last").into(),
            Title::from("duplicate").into(),
            Title::from("Domain Driven Design").into(),
            ScrapKey::with_ctx(&"Test-driven development".into(), &"Book".into()),
            ScrapKey::with_ctx(&"Eric Evans".into(), &"Person".into()),
        ]
        .to_vec();
        result1.sort();
        expected1.sort();
        assert_eq!(result1, expected1);

        // comrak's wikilinks extension rejects multi-pipe forms outright (no
        // wikilink node is emitted). pulldown-cmark accepted the prefix as a key;
        // we lose that case in the swap. Verified separately to document the diff.
        let invalid_links = &[
            "`[[quote block]]`",
            "```\n[[code block]]\n```",
            "[single braces]",
            "only close]]",
            "[[only open",
            "[ [space between brace] ]",
            "[[]]",                                           // empty title
            "[[Test-driven development|TDD|テスト駆動開発]]", // multiple pipes (no wikilink)
        ]
        .join("\n");
        let result2 = scrap_links(invalid_links);

        assert_eq!(result2, Vec::<ScrapKey>::new());
    }
}
