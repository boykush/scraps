use std::collections::HashSet;

use pulldown_cmark::{CowStr, Event, LinkType, Options, Parser, Tag};
use url::Url;

use crate::model::link::ScrapLink;

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

pub fn scrap_links(text: &str) -> Vec<ScrapLink> {
    let parser = Parser::new_ext(text, PARSER_OPTION);

    let links = parser.flat_map(|event| match event {
        Event::Start(Tag::Link {
            link_type: LinkType::WikiLink { has_pothole: _ },
            dest_url: CowStr::Borrowed(dest_url),
            title: _,
            id: _,
        }) => Some(ScrapLink::from_path_str(dest_url)),
        _ => None,
    });

    let hashed: HashSet<ScrapLink> = links.into_iter().collect();
    hashed.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use crate::model::title::Title;

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
            "[[Book/Test-driven development]]",
            "[[Person/Eric Evans|Eric Evans]]",
        ]
        .join("\n");
        let mut result1 = scrap_links(valid_links);
        let mut expected1: Vec<ScrapLink> = [
            Title::from("head").into(),
            Title::from("contain space").into(),
            Title::from("last").into(),
            Title::from("duplicate").into(),
            Title::from("Domain Driven Design").into(),
            Title::from("Test-driven development").into(),
            ScrapLink::with_ctx(&"Test-driven development".into(), &"Book".into()),
            ScrapLink::with_ctx(&"Eric Evans".into(), &"Person".into()),
        ]
        .to_vec();
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
        let result2 = scrap_links(invalid_links);

        assert_eq!(result2, Vec::<ScrapLink>::new());
    }
}
