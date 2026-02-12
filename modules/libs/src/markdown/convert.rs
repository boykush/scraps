use itertools::Itertools;
use pulldown_cmark::{
    html::push_html, CodeBlockKind, CowStr, Event, LinkType, Options, Parser, Tag, TagEnd,
};
use url::Url;

use crate::model::{
    base_url::BaseUrl,
    content::{Content, ContentElement},
    file::ScrapFileStem,
    key::ScrapKey,
    title::Title,
};

const PARSER_OPTION: Options = Options::all();

pub fn to_content(text: &str, base_url: &BaseUrl) -> Content {
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
                let events = handle_wiki_link_events(
                    base_url.as_url(),
                    dest_url,
                    title,
                    id,
                    text,
                    end,
                    has_pothole,
                );
                (0..2).for_each(|_| {
                    parser_windows.next();
                });
                let mut html_buf = String::new();
                push_html(&mut html_buf, events.into_iter());
                content_elements.push(ContentElement::Raw(html_buf))
            }
            (
                Event::Start(Tag::Link {
                    link_type: LinkType::Autolink,
                    dest_url: CowStr::Borrowed(dest_url),
                    title: _,
                    id: _,
                }),
                _,
                _,
            ) => {
                (0..2).for_each(|_| {
                    parser_windows.next();
                });
                match Url::parse(dest_url) {
                    Ok(url) => content_elements.push(ContentElement::Autolink(url)),
                    Err(e) => content_elements
                        .push(ContentElement::Raw(format!("Error parsing URL: {e}"))),
                }
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
    let scrap_link = &ScrapKey::from_path_str(dest_url);
    let file_stem = ScrapFileStem::from(scrap_link.clone());
    let link = format!("{base_url}scraps/{file_stem}.html");
    let start_link = Event::Start(Tag::Link {
        link_type: LinkType::WikiLink { has_pothole },
        dest_url: link.into(),
        title,
        id,
    });
    let replaced_text = if has_pothole {
        text.to_string()
    } else {
        Title::from(scrap_link).to_string()
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
    use rstest::{fixture, rstest};

    #[fixture]
    fn base_url() -> BaseUrl {
        BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap()
    }

    #[rstest]
    #[case::inline_code("`[[quote block]]`", "<p>", "<code>[[quote block]]</code>", "</p>\n")]
    #[case::code_block(
        "```\n[[code block]]\n```",
        "<pre><code>",
        "[[code block]]\n",
        "</code></pre>\n"
    )]
    #[case::bash_block(
        "```bash\nscraps build\n```",
        "<pre><code class=\"language-bash\">",
        "scraps build\n",
        "</code></pre>\n"
    )]
    #[case::mermaid(
        "```mermaid\nflowchart LR\nid\n```",
        "<pre><code class=\"language-mermaid mermaid\">",
        "flowchart LR\nid\n",
        "</code></pre>\n"
    )]
    fn it_to_html_code(
        base_url: BaseUrl,
        #[case] input: &str,
        #[case] expected1: &str,
        #[case] expected2: &str,
        #[case] expected3: &str,
    ) {
        assert_eq!(
            to_content(input, &base_url),
            Content::new(vec![
                ContentElement::Raw(expected1.to_string()),
                ContentElement::Raw(expected2.to_string()),
                ContentElement::Raw(expected3.to_string()),
            ])
        )
    }

    #[rstest]
    #[case::basic(
        "[[link]]",
        "<a href=\"http://localhost:1112/scraps/link.html\">link</a>"
    )]
    #[case::display(
        "[[link|display]]",
        "<a href=\"http://localhost:1112/scraps/link.html\">display</a>"
    )]
    #[case::context(
        "[[Context/link]]",
        "<a href=\"http://localhost:1112/scraps/link.context.html\">link</a>"
    )]
    #[case::context_display(
        "[[Context/link|context display]]",
        "<a href=\"http://localhost:1112/scraps/link.context.html\">context display</a>"
    )]
    #[case::slugify(
        "[[expect slugify]]",
        "<a href=\"http://localhost:1112/scraps/expect-slugify.html\">expect slugify</a>"
    )]
    fn it_to_html_link(base_url: BaseUrl, #[case] input: &str, #[case] expected: &str) {
        assert_eq!(
            to_content(input, &base_url),
            Content::new(vec![
                ContentElement::Raw("<p>".to_string()),
                ContentElement::Raw(expected.to_string()),
                ContentElement::Raw("</p>\n".to_string()),
            ])
        )
    }

    #[rstest]
    #[case::https("<https://example.com>", "https://example.com")]
    #[case::http("<http://example.com>", "http://example.com")]
    fn it_to_html_autolink(base_url: BaseUrl, #[case] input: &str, #[case] expected_url: &str) {
        assert_eq!(
            to_content(input, &base_url),
            Content::new(vec![
                ContentElement::Raw("<p>".to_string()),
                ContentElement::Autolink(Url::parse(expected_url).unwrap()),
                ContentElement::Raw("</p>\n".to_string()),
            ])
        )
    }
}
