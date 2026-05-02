use comrak::{nodes::NodeValue, parse_document, Arena};
use url::Url;

use super::common::options;

pub fn images(text: &str) -> Vec<Url> {
    let arena = Arena::new();
    let opts = options();
    let root = parse_document(&arena, text, &opts);
    root.descendants()
        .filter_map(|node| match &node.data().value {
            NodeValue::Image(node_link) => Url::parse(&node_link.url).ok(),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn url(s: &str) -> Url {
        Url::parse(s).unwrap()
    }

    #[rstest]
    #[case::single(
        "![alt](https://example.com/image.png)",
        vec!["https://example.com/image.png"]
    )]
    #[case::no_image("# header", Vec::<&str>::new())]
    #[case::multiple(
        "![a](https://example.com/a.png) and ![b](https://example.com/b.png)",
        vec!["https://example.com/a.png", "https://example.com/b.png"]
    )]
    #[case::skips_invalid_url(
        "![bad](not-a-url) ![good](https://example.com/g.png)",
        vec!["https://example.com/g.png"]
    )]
    fn it_images(#[case] input: &str, #[case] expected: Vec<&str>) {
        let expected: Vec<Url> = expected.into_iter().map(url).collect();
        assert_eq!(images(input), expected);
    }

    #[test]
    fn it_images_in_code_still_excluded_naturally() {
        assert!(images("`![alt](https://example.com/x.png)`").is_empty());
        assert!(images("```\n![alt](https://example.com/x.png)\n```").is_empty());
    }
}
