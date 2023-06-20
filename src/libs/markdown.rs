use itertools::Itertools;
use pulldown_cmark::{html::push_html, CowStr, Event, LinkType, Parser, Tag};

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

pub fn to_html(text: &str) -> String {
    let mut html_buf = String::new();
    let mut parser = Parser::new(text);

    while let Some(event) = parser.next() {
        match event {
            Event::Text(CowStr::Borrowed("[")) => {
                push_link_events_match_macro(&mut parser, &mut html_buf)
            }
            _ => push_html(&mut html_buf, vec![event].into_iter()),
        }
    }

    html_buf
}

fn push_link_events_match_macro(parser: &mut Parser, html_buf: &mut String) {
    match parser.next_tuple() {
        Some((
            Event::Text(CowStr::Borrowed("[")),
            Event::Text(CowStr::Borrowed(title)),
            Event::Text(CowStr::Borrowed("]")),
            Event::Text(CowStr::Borrowed("]")),
        )) => {
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

            push_html(html_buf, link_events)
        }
        _ => (),
    }
}
