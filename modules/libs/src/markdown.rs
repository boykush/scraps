use std::collections::HashSet;

use super::slugify;
use itertools::Itertools;
use pulldown_cmark::{
    html::push_html, CodeBlockKind, CowStr, Event, LinkType, Options, Parser, Tag,
};
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
                let link_events = to_html_link_events(title, base_url).into_iter();
                // skip next
                (0..4).for_each(|_| {
                    parser_windows.next();
                });
                push_html(&mut html_buf, link_events);
            }
            (&Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref language))), _, _, _, _) => {
                push_html(&mut html_buf, vec![to_html_code_start_event(language)].into_iter())
            }
            (e1, _, _, _, _) => push_html(&mut html_buf, vec![e1.clone()].into_iter()),
        }
    }

    html_buf
}

fn to_html_link_events<'a>(title: &'a str, base_url: &'a Url) -> Vec<Event<'a>> {
    let slug = slugify::by_dash(title);
    let link = format!("{base_url}scraps/{slug}.html");
    let dest_url = CowStr::Boxed(link.into_boxed_str());
    vec![
        Event::Start(Tag::Link {
            link_type: LinkType::Inline,
            dest_url: dest_url.clone(),
            title: CowStr::Borrowed(""),
            id: CowStr::Borrowed(""),
        }),
        Event::Text(CowStr::Borrowed(title)),
        Event::End(
            Tag::Link {
                link_type: LinkType::Inline,
                dest_url: dest_url.clone(),
                title: CowStr::Borrowed(""),
                id: CowStr::Borrowed(""),
            }
            .into(),
        ),
    ]
}

fn to_html_code_start_event<'a>(language: &'a str) -> Event<'a> {
    if language == "mermaid" {
        Event::Html(CowStr::Borrowed(
            // Add the `mermaid`` class in addition to the existing `language-mermaid` class to target it with mermaid.js.
            "<pre><code class=\"language-mermaid mermaid\">",
        ))
    } else {
        Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(CowStr::Borrowed(
            language,
        ))))
    }
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
    fn it_to_html_code() {
        let code_text = [
            "`[[quote block]]`",
            "```\n[[code block]]\n```",
            "```bash\nscraps build\n```",
            "```mermaid\nflowchart LR\nid\n```"
        ]
        .join("\n");
        let base_url = Url::parse("http://localhost:1112/").unwrap();
        let result = to_html(&code_text, &base_url);
        assert_eq!(
            result,
            [
                "<p><code>[[quote block]]</code></p>",
                "<pre><code>[[code block]]\n</code></pre>",
                "<pre><code class=\"language-bash\">scraps build\n</code></pre>",
                "<pre><code class=\"language-mermaid mermaid\">flowchart LR\nid\n</code></pre>"
            ]
            .join("\n")
                + "\n"
        );
    }

    #[test]
    fn it_to_html_link() {
        let base_url = Url::parse("http://localhost:1112/").unwrap();
        let link_text = "[[link]][[expect slugify]]";
        let result1 = to_html(link_text, &base_url);
        assert_eq!(result1, "<p><a href=\"http://localhost:1112/scraps/link.html\">link</a><a href=\"http://localhost:1112/scraps/expect-slugify.html\">expect slugify</a></p>\n",);

        let not_link_text = ["only close]]", "[[only open"].join("\n");
        let result2 = to_html(&not_link_text, &base_url);
        assert_eq!(
            result2,
            ["<p>only close]]", "[[only open</p>",].join("\n") + "\n"
        )
    }
}
