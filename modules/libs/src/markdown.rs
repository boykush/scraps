use std::collections::HashSet;

use super::slugify;
use pulldown_cmark::{
    html::push_html, CodeBlockKind, CowStr, Event, LinkType, Options, Parser, Tag,
};
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

pub fn extract_link_titles(text: &str) -> Vec<String> {
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

pub fn to_html(text: &str, base_url: &Url) -> String {
    let mut html_buf = String::new();
    let parser = Parser::new_ext(text, PARSER_OPTION);

    let replaced = parser.map(|event| match event {
        Event::Start(Tag::Link {
            link_type: LinkType::WikiLink { has_pothole },
            dest_url: CowStr::Borrowed(dest_url),
            title: CowStr::Borrowed(title),
            id,
        }) => {
            let slug = slugify::by_dash(dest_url);
            let link = format!("{base_url}scraps/{slug}.html");
            let start_link = Event::Start(Tag::Link {
                link_type: LinkType::WikiLink { has_pothole },
                dest_url: link.into(),
                title: title.into(),
                id,
            });
            start_link
        }
        Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(CowStr::Borrowed(language)))) => {
            to_html_code_start_event(language)
        }
        e1 => e1.clone(),
    });

    push_html(&mut html_buf, replaced);
    html_buf
}

fn to_html_code_start_event(language: &str) -> Event<'_> {
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
            "[[Domain Driven Design|DDD]]", // alias by pipe
            "[[Test-driven development|TDD|テスト駆動開発]]", // not alias when multiple pipe
        ]
        .join("\n");
        let mut result1 = extract_link_titles(valid_links);
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
        let input_markdown = [
            "[[title]]",
            "[[title|display text]]", // alias by pipe
            "`[[quote block]]`",
            "```\n[[code block]]\n```",
            "```bash\nscraps build\n```",
            "```mermaid\nflowchart LR\nid\n```",
        ]
        .join("\n");
        let base_url = Url::parse("http://localhost:1112/").unwrap();
        let result = to_html(&input_markdown, &base_url);
        assert_eq!(
            result,
            [
                "<p><a href=\"http://localhost:1112/scraps/title.html\">title</a>",
                "<a href=\"http://localhost:1112/scraps/title.html\">display text</a>",
                "<code>[[quote block]]</code></p>",
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
