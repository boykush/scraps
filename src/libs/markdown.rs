use itertools::Itertools;
use pulldown_cmark::{html::push_html, CowStr, Event, LinkType, Parser, Tag};
use url::Url;

pub fn extract_link_titles(text: &str) -> Vec<String> {
    let parser = Parser::new(text);
    let mut parser_windows = parser.tuple_windows();
    let mut link_titles = vec![];

    while let Some(events) = parser_windows.next() {
        match events {
            (
                Event::Text(CowStr::Borrowed("[")),
                Event::Text(CowStr::Borrowed("[")),
                Event::Text(CowStr::Borrowed(title)),
                Event::Text(CowStr::Borrowed("]")),
                Event::Text(CowStr::Borrowed("]")),
            ) => link_titles.push(title.to_string()),
            _ => (),
        }
    }

    link_titles
}

pub fn head_image(text: &str) -> Option<Url> {
    let mut parser = Parser::new(text);
    parser.find_map(|event| match event {
        Event::Start(Tag::Image(_, url, _)) => Url::parse(&url).ok(),
        _ => None,
    })
}

pub fn to_html(text: &str) -> String {
    let mut html_buf = String::new();
    let parser = Parser::new(text);
    let parser_vec = parser.collect::<Vec<Event<'_>>>();
    let mut parser_windows = parser_vec
        .iter()
        .circular_tuple_windows::<(_, _, _, _, _)>()
        .into_iter();

    while let Some(events) = parser_windows.next() {
        match events {
            (
                &Event::Text(CowStr::Borrowed("[")),
                &Event::Text(CowStr::Borrowed("[")),
                &Event::Text(CowStr::Borrowed(title)),
                &Event::Text(CowStr::Borrowed("]")),
                &Event::Text(CowStr::Borrowed("]")),
            ) => {
                let link = &format!("./{}.html", title);
                let link_events = vec![
                    Event::Start(Tag::Link(
                        LinkType::Inline,
                        CowStr::Borrowed(link),
                        CowStr::Borrowed(""),
                    )),
                    Event::Text(CowStr::Borrowed(title)),
                    Event::End(Tag::Link(
                        LinkType::Inline,
                        CowStr::Borrowed(link),
                        CowStr::Borrowed(""),
                    )),
                ]
                .into_iter();

                // skip next
                (0..4).for_each(|_| {
                    parser_windows.next();
                });
                push_html(&mut html_buf, link_events)
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
        let valid_links = &vec!["[[head]]", "[[contain space]]", "[[last]]"].join("\n");
        let result1 = extract_link_titles(valid_links);
        assert_eq!(result1, vec!["head", "contain space", "last"]);

        let invalid_links = &vec![
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
        let code_text = vec!["`[[quote block]]`", "```\n[[code block]]\n```"].join("\n");
        let result1 = to_html(&code_text);
        assert_eq!(
            result1,
            vec![
                "<p><code>[[quote block]]</code></p>",
                "<pre><code>[[code block]]\n</code></pre>",
            ]
            .join("\n")
                + "\n"
        );

        let link_text = "[[link]]";
        let result2 = to_html(&link_text);
        assert_eq!(result2, "<p><a href=\"./link.html\">link</a></p>\n",);

        let not_link_text = vec!["only close]]", "[[only open"].join("\n");
        let result3 = to_html(&not_link_text);
        assert_eq!(
            result3,
            vec!["<p>only close]]", "[[only open</p>",].join("\n") + "\n"
        )
    }
}
