use std::borrow::Cow;

use comrak::{
    format_html,
    nodes::{AstNode, NodeCodeBlock, NodeLink, NodeValue, NodeWikiLink},
    options::Extension,
    parse_document, Arena, Options,
};
use url::Url;

use crate::model::{
    base_url::BaseUrl,
    content::{Content, ContentElement},
    file::ScrapFileStem,
    key::ScrapKey,
    title::Title,
};

fn options() -> Options<'static> {
    Options {
        extension: Extension {
            wikilinks_title_after_pipe: true,
            autolink: true,
            table: true,
            strikethrough: true,
            tasklist: true,
            footnotes: true,
            superscript: true,
            math_dollars: true,
            ..Extension::default()
        },
        ..Options::default()
    }
}

pub fn to_content(text: &str, base_url: &BaseUrl) -> Content {
    let arena = Arena::new();
    let opts = options();
    let root = parse_document(&arena, text, &opts);

    transform_wikilinks(root, base_url);

    let mut elements = Vec::new();
    for child in root.children() {
        push_node(child, &opts, &mut elements);
    }
    Content::new(elements)
}

fn push_node<'a>(node: &'a AstNode<'a>, opts: &Options, out: &mut Vec<ContentElement>) {
    if let Some(url_str) = autolink_paragraph_url(node) {
        out.push(ContentElement::Raw("<p>".to_string()));
        match Url::parse(&url_str) {
            Ok(url) => out.push(ContentElement::Autolink(url)),
            Err(e) => out.push(ContentElement::Raw(format!("Error parsing URL: {e}"))),
        }
        out.push(ContentElement::Raw("</p>\n".to_string()));
        return;
    }

    if let NodeValue::CodeBlock(cb) = &node.data().value {
        let NodeCodeBlock { info, literal, .. } = cb.as_ref();
        if info.split_whitespace().next() == Some("mermaid") {
            let escaped = escape_html(literal);
            out.push(ContentElement::Raw(format!(
                "<pre><code class=\"language-mermaid mermaid\">{escaped}</code></pre>\n"
            )));
            return;
        }
    }

    let mut buf = String::new();
    let _ = format_html(node, opts, &mut buf);
    out.push(ContentElement::Raw(buf));
}

fn autolink_paragraph_url<'a>(node: &'a AstNode<'a>) -> Option<String> {
    if !matches!(node.data().value, NodeValue::Paragraph) {
        return None;
    }
    let mut children = node.children();
    let first = children.next()?;
    if children.next().is_some() {
        return None;
    }
    let NodeValue::Link(link) = &first.data().value else {
        return None;
    };
    let url = link.url.clone();
    let mut link_children = first.children();
    let only_text = link_children.next()?;
    if link_children.next().is_some() {
        return None;
    }
    let NodeValue::Text(t) = &only_text.data().value else {
        return None;
    };
    if t.as_ref() == url.as_str() {
        Some(url)
    } else {
        None
    }
}

fn transform_wikilinks<'a>(root: &'a AstNode<'a>, base_url: &BaseUrl) {
    for node in root.descendants() {
        let url = match &node.data().value {
            NodeValue::WikiLink(NodeWikiLink { url }) => url.clone(),
            _ => continue,
        };

        let scrap_link = ScrapKey::from_path_str(&url);
        let file_stem = ScrapFileStem::from(scrap_link.clone());
        let new_url = format!("{}scraps/{}.html", base_url.as_url(), file_stem);

        let label = collect_text(node);
        let has_pothole = label != url;

        node.data_mut().value = NodeValue::Link(Box::new(NodeLink {
            url: new_url,
            title: String::new(),
        }));

        if !has_pothole {
            let new_label = Title::from(&scrap_link).to_string();
            replace_first_text(node, &new_label);
        }
    }
}

fn collect_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut s = String::new();
    for d in node.descendants() {
        if let NodeValue::Text(t) = &d.data().value {
            s.push_str(t);
        }
    }
    s
}

fn replace_first_text<'a>(node: &'a AstNode<'a>, new_text: &str) {
    for d in node.descendants() {
        if matches!(d.data().value, NodeValue::Text(_)) {
            d.data_mut().value = NodeValue::Text(Cow::Owned(new_text.to_string()));
            return;
        }
    }
}

fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
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
    #[case::inline_code("`[[quote block]]`", "<p><code>[[quote block]]</code></p>\n")]
    #[case::code_block(
        "```\n[[code block]]\n```",
        "<pre><code>[[code block]]\n</code></pre>\n"
    )]
    #[case::bash_block(
        "```bash\nscraps build\n```",
        "<pre><code class=\"language-bash\">scraps build\n</code></pre>\n"
    )]
    #[case::mermaid(
        "```mermaid\nflowchart LR\nid\n```",
        "<pre><code class=\"language-mermaid mermaid\">flowchart LR\nid\n</code></pre>\n"
    )]
    fn it_to_html_code(base_url: BaseUrl, #[case] input: &str, #[case] expected: &str) {
        let content = to_content(input, &base_url);
        assert_eq!(content.to_string(), expected);
    }

    #[rstest]
    #[case::basic(
        "[[link]]",
        "<p><a href=\"http://localhost:1112/scraps/link.html\">link</a></p>\n"
    )]
    #[case::display(
        "[[link|display]]",
        "<p><a href=\"http://localhost:1112/scraps/link.html\">display</a></p>\n"
    )]
    #[case::context(
        "[[Context/link]]",
        "<p><a href=\"http://localhost:1112/scraps/link.context.html\">link</a></p>\n"
    )]
    #[case::context_display(
        "[[Context/link|context display]]",
        "<p><a href=\"http://localhost:1112/scraps/link.context.html\">context display</a></p>\n"
    )]
    #[case::slugify(
        "[[expect slugify]]",
        "<p><a href=\"http://localhost:1112/scraps/expect-slugify.html\">expect slugify</a></p>\n"
    )]
    fn it_to_html_link(base_url: BaseUrl, #[case] input: &str, #[case] expected: &str) {
        let content = to_content(input, &base_url);
        assert_eq!(content.to_string(), expected);
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
