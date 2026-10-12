use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use comrak::{
    Arena, Options, format_html,
    nodes::{AstNode, NodeCodeBlock, NodeLink, NodeValue, NodeWikiLink},
    options::{Extension, Render},
    parse_document,
};
use url::Url;

use crate::markdown::query::{body, resolve_scrap_dest, section};
use crate::model::{
    base_url::BaseUrl,
    content::{Content, ContentElement},
    context::Ctx,
    key::ScrapKey,
    page_path::PagePath,
    scrap::Scrap,
};
use crate::{model::tag::Tag, slugify};

/// The scraps a rendered text may link to or embed: each one's source text,
/// and its title where the scrap has one of its own.
#[derive(Default)]
pub struct LinkTargets {
    texts: HashMap<ScrapKey, String>,
    titles: HashMap<ScrapKey, String>,
}

impl LinkTargets {
    pub fn new<'a>(scraps: impl IntoIterator<Item = &'a Scrap>) -> LinkTargets {
        let mut targets = LinkTargets::default();
        for scrap in scraps {
            let key = scrap.self_key();
            if let Some(title) = scrap.frontmatter_title() {
                targets.titles.insert(key.clone(), title.to_string());
            }
            targets.texts.insert(key, scrap.md_text().to_string());
        }
        targets
    }

    fn contains(&self, key: &ScrapKey) -> bool {
        self.texts.contains_key(key)
    }

    fn text(&self, key: &ScrapKey) -> Option<&str> {
        self.texts.get(key).map(String::as_str)
    }

    fn title(&self, key: &ScrapKey) -> Option<&str> {
        self.titles.get(key).map(String::as_str)
    }
}

pub enum EmbedMode<'a> {
    Expand(&'a LinkTargets),
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
        render: Render {
            // Scraps are authored by the site owner, so raw HTML such as
            // <iframe> embeds is passed through rather than stripped.
            r#unsafe: true,
            ..Render::default()
        },
        ..Options::default()
    }
}

/// Render a scrap's text. `ctx` is the context the text lives in, which is
/// what a relative Markdown link to another scrap's file resolves against.
pub fn to_content(
    text: &str,
    ctx: &Option<Ctx>,
    base_url: &BaseUrl,
    embed_mode: EmbedMode<'_>,
) -> Content {
    to_content_inner(body(text), ctx, base_url, &embed_mode, &mut HashSet::new())
}

fn to_content_inner(
    text: &str,
    ctx: &Option<Ctx>,
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

    let heading_ids = heading_ids(root);
    if let EmbedMode::Expand(targets) = embed_mode {
        transform_md_links(root, ctx, base_url, targets);
    }
    transform_wiki_refs(root, text, base_url, embed_mode, visited_embeds);
    attach_heading_ids(heading_ids, &opts);

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

// Runs before wiki-links become `Link` nodes, so it only ever sees links the
// author wrote as Markdown. One that names no known scrap stays as written.
fn transform_md_links<'a>(
    root: &'a AstNode<'a>,
    ctx: &Option<Ctx>,
    base_url: &BaseUrl,
    targets: &LinkTargets,
) {
    for node in root.descendants() {
        let NodeValue::Link(link) = &mut node.data_mut().value else {
            continue;
        };
        let Some((key, heading)) = resolve_scrap_dest(&link.url, ctx) else {
            continue;
        };
        if targets.contains(&key) {
            link.url = scrap_page_url(base_url, key, heading.as_deref());
        }
    }
}

fn scrap_page_url(base_url: &BaseUrl, key: ScrapKey, heading: Option<&str>) -> String {
    let mut url = PagePath::of_scrap(&key).to_url(base_url);
    if let Some(heading) = heading {
        url.push('#');
        url.push_str(&slugify::by_dash(heading));
    }
    url
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
            if let EmbedMode::Expand(targets) = embed_mode {
                transform_embed(node, base_url, &url, targets, visited_embeds);
            }
            continue;
        }

        let (path, heading) = match url.split_once('#') {
            Some((path, heading)) => (path, Some(heading)),
            None => (url.as_str(), None),
        };
        let scrap_link = ScrapKey::from_path_str(path);
        let new_url = scrap_page_url(base_url, scrap_link.clone(), heading);

        let label = collect_text(node);
        let has_pothole = label != url;

        node.data_mut().value = NodeValue::Link(Box::new(NodeLink {
            url: new_url,
            title: String::new(),
        }));

        // A link written without an alias reads as the scrap's name: its
        // title when the target has one, its stem otherwise.
        if !has_pothole {
            let title = match embed_mode {
                EmbedMode::Expand(targets) => targets.title(&scrap_link),
                EmbedMode::Preserve => None,
            };
            let new_label = title.unwrap_or(scrap_link.stem().as_str());
            replace_first_text(node, new_label);
        }
    }
}

// Ids come from the heading text as authored, before wiki-links are
// relabelled, so they equal the slugs the broken-heading-ref lint compares.
fn heading_ids<'a>(root: &'a AstNode<'a>) -> Vec<(&'a AstNode<'a>, String)> {
    root.descendants()
        .filter(|node| matches!(node.data().value, NodeValue::Heading(_)))
        .map(|node| (node, slugify::by_dash(&collect_text(node))))
        .filter(|(_, id)| !id.is_empty())
        .collect()
}

// comrak's own header ids use its anchorizer, whose slugs differ from
// `slugify::by_dash`, so the id is written into the rendered tag instead.
fn attach_heading_ids<'a>(heading_ids: Vec<(&'a AstNode<'a>, String)>, opts: &Options) {
    for (node, id) in heading_ids {
        let mut html = String::new();
        let _ = format_html(node, opts, &mut html);
        let Some(tag_end) = html.find('>') else {
            continue;
        };
        html.insert_str(tag_end, &format!(" id=\"{}\"", escape_html(&id)));
        detach_children(node);
        node.data_mut().value = NodeValue::Raw(html);
    }
}

fn transform_tag_link<'a>(node: &'a AstNode<'a>, base_url: &BaseUrl, url: &str) {
    let tag_path = url.split_once('#').map_or(url, |(path, _)| path);
    let tag = Tag::from(tag_path);
    if tag.segments().is_empty() {
        return;
    }

    node.data_mut().value = NodeValue::Link(Box::new(NodeLink {
        url: PagePath::of_tag(&tag).to_url(base_url),
        title: String::new(),
    }));
}

fn transform_embed<'a>(
    node: &'a AstNode<'a>,
    base_url: &BaseUrl,
    url: &str,
    targets: &LinkTargets,
    visited_embeds: &mut HashSet<ScrapKey>,
) {
    let (path, heading) = match url.split_once('#') {
        Some((path, heading)) => (path, Some(heading)),
        None => (url, None),
    };
    let scrap_key = ScrapKey::from_path_str(path);

    let Some(text) = targets.text(&scrap_key) else {
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

    // Only a whole scrap can open with a frontmatter block; a section that
    // happens to start with `---` is content.
    let embed_text = heading
        .and_then(|h| section(text, &slugify::by_dash(h)))
        .unwrap_or_else(|| body(text));
    let embedded = to_content_inner(
        embed_text,
        scrap_key.ctx(),
        base_url,
        &EmbedMode::Expand(targets),
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
        let content = to_content(input, &None, &base_url, EmbedMode::Preserve);
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
        "<p><a href=\"http://localhost:1112/scraps/Context/link.html\">link</a></p>\n"
    )]
    #[case::context_display(
        "[[Context/link|context display]]",
        "<p><a href=\"http://localhost:1112/scraps/Context/link.html\">context display</a></p>\n"
    )]
    #[case::nested_context(
        "[[Programming/Rust/borrowing]]",
        "<p><a href=\"http://localhost:1112/scraps/Programming/Rust/borrowing.html\">borrowing</a></p>\n"
    )]
    #[case::space(
        "[[Expect Verbatim]]",
        "<p><a href=\"http://localhost:1112/scraps/Expect%20Verbatim.html\">Expect Verbatim</a></p>\n"
    )]
    #[case::reserved_characters(
        "[[What is it?]]",
        "<p><a href=\"http://localhost:1112/scraps/What%20is%20it%3F.html\">What is it?</a></p>\n"
    )]
    #[case::percent(
        "[[100% sure]]",
        "<p><a href=\"http://localhost:1112/scraps/100%25%20sure.html\">100% sure</a></p>\n"
    )]
    #[case::unicode(
        "[[日本語]]",
        "<p><a href=\"http://localhost:1112/scraps/%E6%97%A5%E6%9C%AC%E8%AA%9E.html\">日本語</a></p>\n"
    )]
    fn it_to_html_link(base_url: BaseUrl, #[case] input: &str, #[case] expected: &str) {
        let content = to_content(input, &None, &base_url, EmbedMode::Preserve);
        assert_eq!(content.to_string(), expected);
    }

    #[rstest]
    #[case::heading(
        "[[Home#Welcome]]",
        "<p><a href=\"http://localhost:1112/scraps/Home.html#welcome\">Home</a></p>\n"
    )]
    #[case::heading_slugify(
        "[[Configuration#SSG section]]",
        "<p><a href=\"http://localhost:1112/scraps/Configuration.html#ssg-section\">Configuration</a></p>\n"
    )]
    #[case::context_heading(
        "[[Ctx/Title#Heading]]",
        "<p><a href=\"http://localhost:1112/scraps/Ctx/Title.html#heading\">Title</a></p>\n"
    )]
    #[case::context_heading_display(
        "[[Ctx/Title#Heading|alias]]",
        "<p><a href=\"http://localhost:1112/scraps/Ctx/Title.html#heading\">alias</a></p>\n"
    )]
    fn it_to_html_heading_link(base_url: BaseUrl, #[case] input: &str, #[case] expected: &str) {
        let content = to_content(input, &None, &base_url, EmbedMode::Preserve);
        assert_eq!(content.to_string(), expected);
    }

    #[rstest]
    #[case::h1("# Welcome", "<h1 id=\"welcome\">Welcome</h1>\n")]
    #[case::spaces("## SSG section", "<h2 id=\"ssg-section\">SSG section</h2>\n")]
    #[case::reserved_char(
        "## Hello, World!",
        "<h2 id=\"hello-comma-world-exclamation\">Hello, World!</h2>\n"
    )]
    #[case::inline_markup(
        "## Hello **bold** world",
        "<h2 id=\"hello-bold-world\">Hello <strong>bold</strong> world</h2>\n"
    )]
    #[case::setext("Title\n=====", "<h1 id=\"title\">Title</h1>\n")]
    #[case::japanese("## 見出し", "<h2 id=\"見出し\">見出し</h2>\n")]
    #[case::nested_in_blockquote(
        "> ## Quoted",
        "<blockquote>\n<h2 id=\"quoted\">Quoted</h2>\n</blockquote>\n"
    )]
    #[case::wikilink(
        "## [[Ctx/topic]]",
        "<h2 id=\"ctx-slash-topic\"><a href=\"http://localhost:1112/scraps/Ctx/topic.html\">topic</a></h2>\n"
    )]
    #[case::attribute_escape("## a \"b\"", "<h2 id=\"a-&quot;b&quot;\">a &quot;b&quot;</h2>\n")]
    fn it_to_html_heading_id(base_url: BaseUrl, #[case] input: &str, #[case] expected: &str) {
        let content = to_content(input, &None, &base_url, EmbedMode::Preserve);
        assert_eq!(content.to_string(), expected);
    }

    #[rstest]
    #[case::single(
        "#[[Markdown]]",
        "<p>#<a href=\"http://localhost:1112/tags/Markdown.html\">Markdown</a></p>\n"
    )]
    #[case::nested(
        "#[[Programming/Rust]]",
        "<p>#<a href=\"http://localhost:1112/tags/Programming/Rust.html\">Programming/Rust</a></p>\n"
    )]
    #[case::space(
        "#[[Domain Driven Design]]",
        "<p>#<a href=\"http://localhost:1112/tags/Domain%20Driven%20Design.html\">Domain Driven Design</a></p>\n"
    )]
    fn it_to_html_tag_link(base_url: BaseUrl, #[case] input: &str, #[case] expected: &str) {
        let content = to_content(input, &None, &base_url, EmbedMode::Preserve);
        assert_eq!(content.to_string(), expected);
    }

    #[test]
    fn it_to_html_embed() {
        let base_url = base_url();
        let scrap_texts = LinkTargets::new(&[scrap("target", "embedded **body**")]);

        let content = to_content(
            "before ![[target]] after",
            &None,
            &base_url,
            EmbedMode::Expand(&scrap_texts),
        );

        assert_eq!(
            content.to_string(),
            "<p>before  <div class=\"scrap-embed\"><p>embedded <strong>body</strong></p>\n</div> after</p>\n"
        );
    }

    fn scrap(key: &str, text: &str) -> Scrap {
        let key = ScrapKey::from_path_str(key);
        Scrap::new(key.stem().as_str(), key.ctx(), text)
    }

    fn wiki(keys: &[&str]) -> Vec<Scrap> {
        keys.iter().map(|key| scrap(key, "")).collect()
    }

    #[rstest]
    #[case::absolute(
        "[customers](/tables/customers.md)",
        "<p><a href=\"http://localhost:1112/scraps/tables/customers.html\">customers</a></p>\n"
    )]
    #[case::relative_with_heading(
        "[revenue](../metrics/revenue.md#Net%20Revenue)",
        "<p><a href=\"http://localhost:1112/scraps/metrics/revenue.html#net-revenue\">revenue</a></p>\n"
    )]
    #[case::sibling_with_space(
        "[**all** orders](./All%20Orders.md)",
        "<p><a href=\"http://localhost:1112/scraps/tables/All%20Orders.html\"><strong>all</strong> orders</a></p>\n"
    )]
    #[case::reference(
        "[customers][c]\n\n[c]: customers.md",
        "<p><a href=\"http://localhost:1112/scraps/tables/customers.html\">customers</a></p>\n"
    )]
    #[case::unresolved("[gone](./gone.md)", "<p><a href=\"./gone.md\">gone</a></p>\n")]
    #[case::escapes_root("[out](../../out.md)", "<p><a href=\"../../out.md\">out</a></p>\n")]
    #[case::scheme(
        "[web](https://example.com/customers.md)",
        "<p><a href=\"https://example.com/customers.md\">web</a></p>\n"
    )]
    #[case::not_markdown(
        "[page](/tables/customers.html)",
        "<p><a href=\"/tables/customers.html\">page</a></p>\n"
    )]
    #[case::image(
        "![shot](./customers.md)",
        "<p><img src=\"./customers.md\" alt=\"shot\" /></p>\n"
    )]
    #[case::inline_code("`[c](./customers.md)`", "<p><code>[c](./customers.md)</code></p>\n")]
    fn it_to_html_markdown_link_to_a_scrap(
        base_url: BaseUrl,
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        let scrap_texts = LinkTargets::new(&wiki(&[
            "tables/customers",
            "tables/All Orders",
            "metrics/revenue",
        ]));
        let content = to_content(
            input,
            &Some("tables".into()),
            &base_url,
            EmbedMode::Expand(&scrap_texts),
        );
        assert_eq!(content.to_string(), expected);
    }

    #[rstest]
    fn it_gives_a_markdown_link_the_url_of_the_wiki_link(base_url: BaseUrl) {
        let scrap_texts = LinkTargets::new(&wiki(&["metrics/revenue"]));
        let render = |input: &str| {
            to_content(input, &None, &base_url, EmbedMode::Expand(&scrap_texts)).to_string()
        };
        assert_eq!(
            render("[revenue](/metrics/revenue.md#definition)"),
            render("[[metrics/revenue#definition|revenue]]")
        );
    }

    #[rstest]
    fn it_resolves_a_markdown_link_from_the_embedded_scrap(base_url: BaseUrl) {
        let scrap_texts = LinkTargets::new(&[
            scrap("tables/customers", ""),
            scrap("tables/orders", "[c](./customers.md)"),
        ]);
        let content = to_content(
            "![[tables/orders]]",
            &None,
            &base_url,
            EmbedMode::Expand(&scrap_texts),
        );
        assert!(
            content
                .to_string()
                .contains("href=\"http://localhost:1112/scraps/tables/customers.html\"")
        );
    }

    #[rstest]
    #[case::unaliased(
        "[[tables/orders]]",
        "<p><a href=\"http://localhost:1112/scraps/tables/orders.html\">Customer Orders</a></p>\n"
    )]
    #[case::unaliased_with_heading(
        "[[tables/orders#Columns]]",
        "<p><a href=\"http://localhost:1112/scraps/tables/orders.html#columns\">Customer Orders</a></p>\n"
    )]
    #[case::alias_kept(
        "[[tables/orders|the orders table]]",
        "<p><a href=\"http://localhost:1112/scraps/tables/orders.html\">the orders table</a></p>\n"
    )]
    #[case::markdown_link_text_kept(
        "[orders](/tables/orders.md)",
        "<p><a href=\"http://localhost:1112/scraps/tables/orders.html\">orders</a></p>\n"
    )]
    #[case::untitled_target_reads_as_its_stem(
        "[[tables/customers]]",
        "<p><a href=\"http://localhost:1112/scraps/tables/customers.html\">customers</a></p>\n"
    )]
    #[case::missing_target_reads_as_its_stem(
        "[[tables/gone]]",
        "<p><a href=\"http://localhost:1112/scraps/tables/gone.html\">gone</a></p>\n"
    )]
    #[case::a_title_is_not_a_link_target(
        "[[Customer Orders]]",
        "<p><a href=\"http://localhost:1112/scraps/Customer%20Orders.html\">Customer Orders</a></p>\n"
    )]
    fn it_labels_an_unaliased_wiki_link_with_the_title(
        base_url: BaseUrl,
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        let targets = LinkTargets::new(&[
            scrap("tables/orders", "---\ntitle: Customer Orders\n---\n"),
            scrap("tables/customers", ""),
        ]);
        let content = to_content(input, &None, &base_url, EmbedMode::Expand(&targets));
        assert_eq!(content.to_string(), expected);
    }

    #[test]
    fn it_to_html_embed_section() {
        let base_url = base_url();
        let scrap_texts =
            LinkTargets::new(&[scrap("target", "## Keep\n\nwanted\n\n## Skip\n\nignored\n")]);

        let content = to_content(
            "![[target#Keep]]",
            &None,
            &base_url,
            EmbedMode::Expand(&scrap_texts),
        );

        assert!(content.to_string().contains("wanted"));
        assert!(!content.to_string().contains("ignored"));
    }

    #[rstest]
    #[case::iframe(
        "<iframe src=\"https://example.com\"></iframe>",
        "<iframe src=\"https://example.com\"></iframe>\n"
    )]
    #[case::inline_html("text with <br> break", "<p>text with <br> break</p>\n")]
    fn it_to_html_raw_html(base_url: BaseUrl, #[case] input: &str, #[case] expected: &str) {
        let content = to_content(input, &None, &base_url, EmbedMode::Preserve);
        assert_eq!(content.to_string(), expected);
    }

    #[rstest]
    #[case::https("<https://example.com>", "https://example.com")]
    #[case::http("<http://example.com>", "http://example.com")]
    fn it_to_html_autolink(base_url: BaseUrl, #[case] input: &str, #[case] expected_url: &str) {
        assert_eq!(
            to_content(input, &None, &base_url, EmbedMode::Preserve),
            Content::new(vec![
                ContentElement::Raw("<p>".to_string()),
                ContentElement::Autolink(Url::parse(expected_url).unwrap()),
                ContentElement::Raw("</p>\n".to_string()),
            ])
        )
    }

    const FRONTED: &str =
        "---\ntype: BigQuery Table\ntitle: Customer Orders\n---\n\n# Schema\n\nBody text.\n";

    #[rstest]
    fn it_to_html_without_the_frontmatter_block(base_url: BaseUrl) {
        let content = to_content(FRONTED, &None, &base_url, EmbedMode::Preserve);
        assert_eq!(
            content.to_string(),
            "<h1 id=\"schema\">Schema</h1>\n<p>Body text.</p>\n"
        );
    }

    #[rstest]
    #[case::unclosed("---\ntype: Blog\n\n# Schema\n")]
    #[case::malformed("---\ntype: [Blog\n---\n\n# Schema\n")]
    fn it_to_html_keeps_an_unrecognised_block(base_url: BaseUrl, #[case] input: &str) {
        let content = to_content(input, &None, &base_url, EmbedMode::Preserve).to_string();
        assert!(content.starts_with("<hr />\n"), "{content}");
    }

    #[rstest]
    #[case::whole("![[target]]")]
    #[case::missing_heading("![[target#Nowhere]]")]
    fn it_to_html_embed_without_the_frontmatter_block(base_url: BaseUrl, #[case] input: &str) {
        let scrap_texts = LinkTargets::new(&[scrap("target", FRONTED)]);

        let content = to_content(input, &None, &base_url, EmbedMode::Expand(&scrap_texts));

        assert_eq!(
            content.to_string(),
            "<p><div class=\"scrap-embed\"><h1 id=\"schema\">Schema</h1>\n<p>Body text.</p>\n</div></p>\n"
        );
    }

    #[rstest]
    fn it_to_html_embed_section_starting_with_a_rule(base_url: BaseUrl) {
        let scrap_texts = LinkTargets::new(&[scrap("target", "# Keep\n---\nkey: value\n---\n")]);

        let content = to_content(
            "![[target#Keep]]",
            &None,
            &base_url,
            EmbedMode::Expand(&scrap_texts),
        );

        assert!(content.to_string().contains("key: value"));
    }
}
