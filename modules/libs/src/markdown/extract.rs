use std::collections::HashSet;

use pulldown_cmark::{CowStr, Event, LinkType, Options, Parser, Tag};
use url::Url;

use crate::model::key::ScrapKey;

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

pub fn scrap_tags(text: &str) -> Vec<ScrapKey> {
    let parser = Parser::new_ext(text, PARSER_OPTION);
    let events: Vec<Event> = parser.collect();

    let mut tags = Vec::new();
    for i in 0..events.len() {
        if let Event::Start(Tag::Link {
            link_type: LinkType::WikiLink { has_pothole: _ },
            dest_url: CowStr::Borrowed(dest_url),
            title: _,
            id: _,
        }) = &events[i]
        {
            // Check if the preceding text event ends with '#'
            if i > 0 {
                if let Event::Text(prev_text) = &events[i - 1] {
                    if prev_text.ends_with('#') {
                        tags.push(ScrapKey::from_path_str(dest_url));
                    }
                }
            }
        }
    }

    let hashed: HashSet<ScrapKey> = tags.into_iter().collect();
    hashed.into_iter().collect()
}

pub fn scrap_links(text: &str) -> Vec<ScrapKey> {
    let parser = Parser::new_ext(text, PARSER_OPTION);

    let links = parser.flat_map(|event| match event {
        Event::Start(Tag::Link {
            link_type: LinkType::WikiLink { has_pothole: _ },
            dest_url: CowStr::Borrowed(dest_url),
            title: _,
            id: _,
        }) => Some(ScrapKey::from_path_str(dest_url)),
        _ => None,
    });

    let hashed: HashSet<ScrapKey> = links.into_iter().collect();
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
    fn it_hash_wiki_link_events() {
        // Verify pulldown-cmark event sequence for #[[tag]] syntax
        let text = "text #[[tag1]] more";
        let parser = Parser::new_ext(text, PARSER_OPTION);
        let events: Vec<Event> = parser.collect();

        // Expect: Paragraph Start, Text("text #"), WikiLink Start, Text("tag1"), WikiLink End, Text(" more"), Paragraph End
        let has_hash_before_wikilink = events.windows(2).any(|w| {
            matches!(&w[0], Event::Text(t) if t.ends_with('#'))
                && matches!(
                    &w[1],
                    Event::Start(Tag::Link {
                        link_type: LinkType::WikiLink { .. },
                        ..
                    })
                )
        });
        assert!(
            has_hash_before_wikilink,
            "Expected Text ending with '#' before WikiLink Start. Events: {events:?}"
        );

        // Verify line-initial #[[tag]] does NOT become a heading (no space after #)
        let text_line_start = "#[[tag2]]";
        let parser2 = Parser::new_ext(text_line_start, PARSER_OPTION);
        let events2: Vec<Event> = parser2.collect();
        let has_heading = events2
            .iter()
            .any(|e| matches!(e, Event::Start(Tag::Heading { .. })));
        assert!(
            !has_heading,
            "Line-initial #[[tag]] should NOT be parsed as heading. Events: {events2:?}"
        );

        // Verify #[[tag]] is still parsed as WikiLink when at line start
        let has_wikilink = events2.iter().any(|e| {
            matches!(
                e,
                Event::Start(Tag::Link {
                    link_type: LinkType::WikiLink { .. },
                    ..
                })
            )
        });
        assert!(
            has_wikilink,
            "Line-initial #[[tag]] should still produce WikiLink. Events: {events2:?}"
        );
    }

    #[test]
    fn it_scrap_tags() {
        // #[[tag]] should be extracted as tags, [[link]] should not
        let text = "#[[tag1]] some text #[[tag2]] [[link1]]";
        let mut result = scrap_tags(text);
        let mut expected: Vec<ScrapKey> =
            vec![Title::from("tag1").into(), Title::from("tag2").into()];
        result.sort();
        expected.sort();
        assert_eq!(result, expected);

        // Only [[link]] without # prefix should not be extracted
        let text_no_tags = "[[link1]] [[link2]]";
        let result2 = scrap_tags(text_no_tags);
        assert_eq!(result2, Vec::<ScrapKey>::new());

        // Line-initial #[[tag]] should work
        let text_line_start = "#[[tag3]]";
        let result3 = scrap_tags(text_line_start);
        assert_eq!(result3, vec![ScrapKey::from(Title::from("tag3"))]);

        // Context path in tag: #[[Context/tag]]
        let text_ctx = "#[[Book/Rust]]";
        let result4 = scrap_tags(text_ctx);
        assert_eq!(
            result4,
            vec![ScrapKey::with_ctx(&"Rust".into(), &"Book".into())]
        );

        // Duplicates should be deduplicated
        let text_dup = "#[[dup]] #[[dup]]";
        let result5 = scrap_tags(text_dup);
        assert_eq!(result5.len(), 1);
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
            "[[Test-driven development|TDD|テスト駆動開発]]", // not alias when multiple pipe
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
            Title::from("Test-driven development").into(),
            ScrapKey::with_ctx(&"Test-driven development".into(), &"Book".into()),
            ScrapKey::with_ctx(&"Eric Evans".into(), &"Person".into()),
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

        assert_eq!(result2, Vec::<ScrapKey>::new());
    }
}
