use std::collections::HashSet;

use pulldown_cmark::{CowStr, Event, LinkType, Options, Parser, Tag};
use url::Url;

const PARSER_OPTION: Options = Options::all();

pub fn head_image(text: &str) -> Option<Url> {
    let mut parser = Parser::new_ext(text, PARSER_OPTION);
    parser.find_map(|event| match event {
        Event::Start(Tag::Image {
            link_type: _,
            dest_url,
            title: _,
            id: _,
        }) => Url::parse(&dest_url).ok(),
        _ => None,
    })
}

pub fn link_titles(text: &str) -> Vec<String> {
    let parser = Parser::new_ext(text, PARSER_OPTION);

    let link_titles = parser.flat_map(|event| match event {
        Event::Start(Tag::Link {
            link_type: LinkType::WikiLink { has_pothole: _ },
            dest_url: CowStr::Borrowed(dest_url),
            title: _,
            id: _,
        }) => Some(dest_url.to_string()),
        _ => None,
    });

    let hashed: HashSet<String> = link_titles.into_iter().collect();
    hashed.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_head_image() {
        assert_eq!(
            head_image("![alt](https://example.com/image.png)"),
            Some(Url::parse("https://example.com/image.png").unwrap())
        );
        assert_eq!(head_image("# header1"), None)
    }

    #[test]
    fn it_link_titles() {
        let valid_links = &[
            "[[head]]",
            "[[contain space]]",
            "[[last]]",
            "[[duplicate]]",
            "[[duplicate]]",
            "[[Domain Driven Design|DDD]]", // alias by pipe
            "[[Test-driven development|TDD|テスト駆動開発]]", // not alias when multiple pipe
        ]
        .join("\n");
        let mut result1 = link_titles(valid_links);
        let mut expected1 = [
            "head",
            "contain space",
            "last",
            "duplicate",
            "Domain Driven Design",
            "Test-driven development",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
        result1.sort();
        expected1.sort();
        assert_eq!(result1, expected1);

        let invalid_links = &[
            "`[[quote block]]`",
            "```\n[[code block]]\n```",
            "[single braces]",
            "only close]]",
            "[[only open",
            "[ [space between brace] ]",
            "[[]]", // empty title
        ]
        .join("\n");
        let result2 = link_titles(invalid_links);

        assert_eq!(result2, Vec::<&str>::new());
    }
}
