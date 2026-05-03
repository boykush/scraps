use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

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
use crate::{model::tag::Tag, slugify};

pub enum EmbedMode<'a> {
    Expand(&'a HashMap<ScrapKey, String>),
    Preserve,
}

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

pub fn to_content(text: &str, base_url: &BaseUrl, embed_mode: EmbedMode<'_>) -> Content {
    to_content_inner(text, base_url, &embed_mode, &mut HashSet::new())
}

fn to_content_inner(
    text: &str,
    base_url: &BaseUrl,
    embed_mode: &EmbedMode<'_>,
    visited_embeds: &mut HashSet<ScrapKey>,
) -> Content {
    let arena = Arena::new();
    let opts = options();
    let parse_text = match embed_mode {
        EmbedMode::Expand(_) => expose_embed_wikilinks(text),
        EmbedMode::Preserve => Cow::Borrowed(text),
    };
    let root = parse_document(&arena, &parse_text, &opts);

    transform_wiki_refs(root, text, base_url, embed_mode, visited_embeds);

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

fn transform_wiki_refs<'a>(
    root: &'a AstNode<'a>,
    source_text: &str,
    base_url: &BaseUrl,
    embed_mode: &EmbedMode<'_>,
    visited_embeds: &mut HashSet<ScrapKey>,
) {
    let starts = line_starts(source_text);
    let wiki_nodes: Vec<_> = root
        .descendants()
        .filter(|node| matches!(node.data().value, NodeValue::WikiLink(_)))
        .collect();
    for node in wiki_nodes {
        let url = match &node.data().value {
            NodeValue::WikiLink(NodeWikiLink { url }) => url.clone(),
            _ => continue,
        };

        let pos = node.data().sourcepos;
        let byte = line_col_to_byte(&starts, pos.start.line, pos.start.column);
        let prefix = byte
            .checked_sub(1)
            .and_then(|i| source_text.as_bytes().get(i));

        if prefix == Some(&b'#') {
            transform_tag_link(node, base_url, &url);
            continue;
        }

        if prefix == Some(&b'!') {
            if let EmbedMode::Expand(scrap_texts) = embed_mode {
                transform_embed(node, base_url, &url, scrap_texts, visited_embeds);
            }
            continue;
        }

        let scrap_link = ScrapKey::from_path_str(&url);
        let file_stem = ScrapFileStem::from(scrap_link.clone());
        let mut new_url = format!("{}scraps/{}.html", base_url.as_url(), file_stem);
        if let Some((_, heading)) = url.split_once('#') {
            new_url.push('#');
            new_url.push_str(&slugify::by_dash(heading));
        }

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

fn transform_tag_link<'a>(node: &'a AstNode<'a>, base_url: &BaseUrl, url: &str) {
    let tag_path = url.split_once('#').map_or(url, |(path, _)| path);
    let tag = Tag::from(tag_path);
    let slug = tag
        .segments()
        .iter()
        .map(|seg| slugify::by_dash(seg))
        .collect::<Vec<_>>()
        .join("/");
    if slug.is_empty() {
        return;
    }

    node.data_mut().value = NodeValue::Link(Box::new(NodeLink {
        url: format!("{}tags/{}.html", base_url.as_url(), slug),
        title: String::new(),
    }));
}

fn transform_embed<'a>(
    node: &'a AstNode<'a>,
    base_url: &BaseUrl,
    url: &str,
    scrap_texts: &HashMap<ScrapKey, String>,
    visited_embeds: &mut HashSet<ScrapKey>,
) {
    let (path, heading) = match url.split_once('#') {
        Some((path, heading)) => (path, Some(heading)),
        None => (url, None),
    };
    let scrap_key = ScrapKey::from_path_str(path);

    let Some(text) = scrap_texts.get(&scrap_key) else {
        detach_children(node);
        node.data_mut().value = NodeValue::Raw(format!(
            "<div class=\"scrap-embed scrap-embed-missing\">{}</div>",
            escape_html(path)
        ));
        return;
    };

    if !visited_embeds.insert(scrap_key.clone()) {
        detach_children(node);
        node.data_mut().value = NodeValue::Raw(format!(
            "<div class=\"scrap-embed scrap-embed-cycle\">{}</div>",
            escape_html(path)
        ));
        return;
    }

    let embed_text = heading
        .and_then(|h| crate::markdown::query::section(text, &slugify::by_dash(h)))
        .unwrap_or(text);
    let embedded = to_content_inner(
        embed_text,
        base_url,
        &EmbedMode::Expand(scrap_texts),
        visited_embeds,
    );
    visited_embeds.remove(&scrap_key);

    detach_children(node);
    node.data_mut().value =
        NodeValue::Raw(format!("<div class=\"scrap-embed\">{}</div>", embedded));
}

fn detach_children<'a>(node: &'a AstNode<'a>) {
    let children: Vec<_> = node.children().collect();
    for child in children {
        child.detach();
    }
}

fn expose_embed_wikilinks(text: &str) -> Cow<'_, str> {
    if !text.contains("![[") {
        return Cow::Borrowed(text);
    }

    let bytes = text.as_bytes();
    let mut out = bytes.to_vec();
    let mut i = 0;
    while i + 2 < bytes.len() {
        if bytes[i] == b'!' && bytes[i + 1] == b'[' && bytes[i + 2] == b'[' {
            out[i] = b' ';
        }
        i += 1;
    }
    String::from_utf8(out)
        .map(Cow::Owned)
        .unwrap_or_else(|_| Cow::Borrowed(text))
}

fn line_starts(text: &str) -> Vec<usize> {
    let mut v = vec![0];
    for (i, b) in text.bytes().enumerate() {
        if b == b'\n' {
            v.push(i + 1);
        }
    }
    v
}

fn line_col_to_byte(starts: &[usize], line: usize, col: usize) -> usize {
    let li = line.saturating_sub(1);
    let base = starts.get(li).copied().unwrap_or(0);
    base + col.saturating_sub(1)
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
    use std::collections::HashMap;

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
        let content = to_content(input, &base_url, EmbedMode::Preserve);
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
        "<p><a href=\"http://localhost:1112/scraps/context/link.html\">link</a></p>\n"
    )]
    #[case::context_display(
        "[[Context/link|context display]]",
        "<p><a href=\"http://localhost:1112/scraps/context/link.html\">context display</a></p>\n"
    )]
    #[case::nested_context(
        "[[Programming/Rust/borrowing]]",
        "<p><a href=\"http://localhost:1112/scraps/programming/rust/borrowing.html\">borrowing</a></p>\n"
    )]
    #[case::slugify(
        "[[expect slugify]]",
        "<p><a href=\"http://localhost:1112/scraps/expect-slugify.html\">expect slugify</a></p>\n"
    )]
    fn it_to_html_link(base_url: BaseUrl, #[case] input: &str, #[case] expected: &str) {
        let content = to_content(input, &base_url, EmbedMode::Preserve);
        assert_eq!(content.to_string(), expected);
    }

    #[rstest]
    #[case::single(
        "#[[Markdown]]",
        "<p>#<a href=\"http://localhost:1112/tags/markdown.html\">Markdown</a></p>\n"
    )]
    #[case::nested("#[[Programming/Rust]]", "<p>#<a href=\"http://localhost:1112/tags/programming/rust.html\">Programming/Rust</a></p>\n")]
    fn it_to_html_tag_link(base_url: BaseUrl, #[case] input: &str, #[case] expected: &str) {
        let content = to_content(input, &base_url, EmbedMode::Preserve);
        assert_eq!(content.to_string(), expected);
    }

    #[test]
    fn it_to_html_embed() {
        let base_url = base_url();
        let mut scrap_texts = HashMap::new();
        scrap_texts.insert(
            ScrapKey::from_path_str("target"),
            "embedded **body**".to_string(),
        );

        let content = to_content(
            "before ![[target]] after",
            &base_url,
            EmbedMode::Expand(&scrap_texts),
        );

        assert_eq!(
            content.to_string(),
            "<p>before  <div class=\"scrap-embed\"><p>embedded <strong>body</strong></p>\n</div> after</p>\n"
        );
    }

    #[test]
    fn it_to_html_embed_section() {
        let base_url = base_url();
        let mut scrap_texts = HashMap::new();
        scrap_texts.insert(
            ScrapKey::from_path_str("target"),
            "## Keep\n\nwanted\n\n## Skip\n\nignored\n".to_string(),
        );

        let content = to_content(
            "![[target#Keep]]",
            &base_url,
            EmbedMode::Expand(&scrap_texts),
        );

        assert!(content.to_string().contains("wanted"));
        assert!(!content.to_string().contains("ignored"));
    }

    #[rstest]
    #[case::https("<https://example.com>", "https://example.com")]
    #[case::http("<http://example.com>", "http://example.com")]
    fn it_to_html_autolink(base_url: BaseUrl, #[case] input: &str, #[case] expected_url: &str) {
        assert_eq!(
            to_content(input, &base_url, EmbedMode::Preserve),
            Content::new(vec![
                ContentElement::Raw("<p>".to_string()),
                ContentElement::Autolink(Url::parse(expected_url).unwrap()),
                ContentElement::Raw("</p>\n".to_string()),
            ])
        )
    }
}
