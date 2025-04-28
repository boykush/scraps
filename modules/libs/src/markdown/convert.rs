use itertools::Itertools;
use pulldown_cmark::{
    html::push_html, CodeBlockKind, CowStr, Event, LinkType, Options, Parser, Tag, TagEnd,
};
use url::Url;

use crate::model::{
    content::{Content, ContentElement},
    file::ScrapFileStem,
    link::ScrapLink,
};

const PARSER_OPTION: Options = Options::all();

pub fn to_html_content(text: &str, base_url: &Url) -> Content {
    let parser = Parser::new_ext(text, PARSER_OPTION);
    let parser_vec = parser.into_iter().collect::<Vec<_>>();
    let mut parser_windows = parser_vec.into_iter().circular_tuple_windows::<(_, _, _)>();
    let mut content_elements = Vec::new();

    while let Some(events) = parser_windows.next() {
        match events {
            (
                Event::Start(Tag::Link {
                    link_type: LinkType::WikiLink { has_pothole },
                    dest_url: CowStr::Borrowed(dest_url),
                    title,
                    id,
                }),
                Event::Text(CowStr::Borrowed(text)),
                end @ Event::End(TagEnd::Link),
            ) => {
                let events =
                    handle_wiki_link_events(base_url, dest_url, title, id, text, end, has_pothole);
                (0..2).for_each(|_| {
                    parser_windows.next();
                });
                let mut html_buf = String::new();
                push_html(&mut html_buf, events.into_iter());
                content_elements.push(ContentElement::Raw(html_buf))
            }
            (
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(CowStr::Borrowed(language)))),
                _,
                _,
            ) => {
                let mut html_buf = String::new();
                push_html(
                    &mut html_buf,
                    [handle_code_block_start_event(language)].into_iter(),
                );
                content_elements.push(ContentElement::Raw(html_buf))
            }
            (e1, _, _) => {
                let mut html_buf = String::new();
                push_html(&mut html_buf, [e1].into_iter());
                content_elements.push(ContentElement::Raw(html_buf))
            }
        }
    }
    Content::new(content_elements)
}

fn handle_wiki_link_events<'a>(
    base_url: &Url,
    dest_url: &str,
    title: CowStr<'a>,
    id: CowStr<'a>,
    text: &str,
    end: Event<'a>,
    has_pothole: bool,
) -> [Event<'a>; 3] {
    let scrap_link = &ScrapLink::from_path_str(dest_url);
    let file_stem = ScrapFileStem::from(scrap_link.clone());
    let link = format!("{}scraps/{}.html", base_url, file_stem);
    let start_link = Event::Start(Tag::Link {
        link_type: LinkType::WikiLink { has_pothole },
        dest_url: link.into(),
        title,
        id,
    });
    let replaced_text = if has_pothole {
        text.to_string()
    } else {
        scrap_link.title.to_string()
    };
    [start_link, Event::Text(replaced_text.into()), end]
}

fn handle_code_block_start_event(language: &str) -> Event<'_> {
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
    fn it_to_html_code() {
        let input_list = [
            "`[[quote block]]`",
            "```\n[[code block]]\n```",
            "```bash\nscraps build\n```",
            "```mermaid\nflowchart LR\nid\n```",
        ];
        let expected_list = [
            "<p><code>[[quote block]]</code></p>",
            "<pre><code>[[code block]]\n</code></pre>",
            "<pre><code class=\"language-bash\">scraps build\n</code></pre>",
            "<pre><code class=\"language-mermaid mermaid\">flowchart LR\nid\n</code></pre>",
        ];
        let base_url = Url::parse("http://localhost:1112/").unwrap();
        input_list
            .iter()
            .zip(expected_list)
            .for_each(|(input, expected)| {
                assert_eq!(
                    to_html_content(input, &base_url),
                    Content::new(vec![ContentElement::Raw(expected.to_string() + "\n")])
                )
            });
    }

    #[test]
    fn it_to_html_link() {
        let base_url = Url::parse("http://localhost:1112/").unwrap();
        let input_list = [
            "[[link]]",
            "[[link|display]]",
            "[[Context/link]]",
            "[[Context/link|context display]]",
            "[[expect slugify]]",
        ];
        let expected_list = [
            "<a href=\"http://localhost:1112/scraps/link.html\">link</a>",
            "<a href=\"http://localhost:1112/scraps/link.html\">display</a>",
            "<a href=\"http://localhost:1112/scraps/link.context.html\">link</a>",
            "<a href=\"http://localhost:1112/scraps/link.context.html\">context display</a>",
            "<a href=\"http://localhost:1112/scraps/expect-slugify.html\">expect slugify</a>",
        ];
        input_list
            .iter()
            .zip(expected_list)
            .for_each(|(input, expected)| {
                assert_eq!(
                    to_html_content(input, &base_url),
                    Content::new(vec![ContentElement::Raw(
                        "<p>".to_string() + expected + "</p>\n"
                    )])
                )
            });
    }
}
