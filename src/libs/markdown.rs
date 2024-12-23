use std::collections::HashSet;

use itertools::Itertools;
use pulldown_cmark::{html::push_html, CowStr, Event, LinkType, Options, Parser, Tag};
use scraps_libs::slugify;
use url::Url;

const PARSER_OPTION: Options = Options::all();

pub fn extract_link_titles(text: &str) -> Vec<String> {
    let parser = Parser::new_ext(text, PARSER_OPTION);
    let parser_windows = parser.tuple_windows();
    let mut link_titles = vec![];

    for events in parser_windows {
        if let (
            Event::Text(CowStr::Borrowed("[")),
            Event::Text(CowStr::Borrowed("[")),
            Event::Text(CowStr::Borrowed(title)),
            Event::Text(CowStr::Borrowed("]")),
            Event::Text(CowStr::Borrowed("]")),
        ) = events
        {
            link_titles.push(title.to_string());
        }
    }

    let hashed: HashSet<String> = link_titles.into_iter().collect();
    hashed.into_iter().collect()
}

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

pub fn to_html(text: &str, base_url: &Url) -> String {
    let mut html_buf = String::new();
    let parser = Parser::new_ext(text, PARSER_OPTION);
    let parser_vec = parser.collect::<Vec<Event<'_>>>();
    let mut parser_windows = parser_vec
        .iter()
        .circular_tuple_windows::<(_, _, _, _, _)>();

    while let Some(events) = parser_windows.next() {
        match events {
            (
                &Event::Text(CowStr::Borrowed("[")),
                &Event::Text(CowStr::Borrowed("[")),
                &Event::Text(CowStr::Borrowed(title)),
                &Event::Text(CowStr::Borrowed("]")),
                &Event::Text(CowStr::Borrowed("]")),
            ) => {
                let slug = slugify::by_dash(title);
                let link = &format!("{base_url}scraps/{slug}.html");
                let link_events = vec![
                    Event::Start(Tag::Link {
                        link_type: LinkType::Inline,
                        dest_url: CowStr::Borrowed(link),
                        title: CowStr::Borrowed(""),
                        id: CowStr::Borrowed(""),
                    }),
                    Event::Text(CowStr::Borrowed(title)),
                    Event::End(
                        Tag::Link {
                            link_type: LinkType::Inline,
                            dest_url: CowStr::Borrowed(link),
                            title: CowStr::Borrowed(""),
                            id: CowStr::Borrowed(""),
                        }
                        .into(),
                    ),
                ]
                .into_iter();

                // skip next
                (0..4).for_each(|_| {
                    parser_windows.next();
                });
                push_html(&mut html_buf, link_events);
            }
            (e1, _, _, _, _) => push_html(&mut html_buf, vec![e1.clone()].into_iter()),
        }
    }

    html_buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_extract_link_titles() {
        let valid_links = &[
            "[[head]]",
            "[[contain space]]",
            "[[last]]",
            "[[duplicate]]",
            "[[duplicate]]",
        ]
        .join("\n");
        let mut result1 = extract_link_titles(valid_links);
        let mut expected1 = ["head", "contain space", "last", "duplicate"];
        result1.sort();
        expected1.sort();
        assert_eq!(result1, expected1);

        let invalid_links = &[
            "`[[quote block]]`",
            "```\n[[code block]]\n```",
            "[single braces]",
            "[[contain\nbreak]]",
            "only close]]",
            "[[only open",
            "[ [space between brace] ]",
            "[[]]", // empty title
        ]
        .join("\n");
        let result2 = extract_link_titles(invalid_links);

        assert_eq!(result2, Vec::<&str>::new());
    }

    #[test]
    fn it_head_image() {
        assert_eq!(
            head_image("![alt](https://example.com/image.png)"),
            Some(Url::parse("https://example.com/image.png").unwrap())
        );
        assert_eq!(head_image("# header1"), None)
    }

    #[test]
    fn it_to_html() {
        let code_text = ["`[[quote block]]`", "```\n[[code block]]\n```"].join("\n");
        let base_url = Url::parse("http://localhost:1112/").unwrap();
        let result1 = to_html(&code_text, &base_url);
        assert_eq!(
            result1,
            [
                "<p><code>[[quote block]]</code></p>",
                "<pre><code>[[code block]]\n</code></pre>",
            ]
            .join("\n")
                + "\n"
        );

        let link_text = "[[link]][[expect slugify]]";
        let result2 = to_html(link_text, &base_url);
        assert_eq!(result2, "<p><a href=\"http://localhost:1112/scraps/link.html\">link</a><a href=\"http://localhost:1112/scraps/expect-slugify.html\">expect slugify</a></p>\n",);

        let not_link_text = ["only close]]", "[[only open"].join("\n");
        let result3 = to_html(&not_link_text, &base_url);
        assert_eq!(
            result3,
            ["<p>only close]]", "[[only open</p>",].join("\n") + "\n"
        )
    }
}
