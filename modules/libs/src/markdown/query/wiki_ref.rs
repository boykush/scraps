//! Unified retrieval of wiki-shaped inline syntax: `[[link]]`, `#[[tag]]`,
//! and `![[embed]]` are all variants of the same `[[]]` family. This module
//! lets comrak extract the bracketed shape first, then classifies each
//! occurrence by the prefix immediately attached to the `[[`. A Markdown
//! link to a `.md` path is read alongside them as one more reference.

use comrak::{
    Arena,
    nodes::{NodeLink, NodeValue, NodeWikiLink},
    parse_document,
};

use super::common::{collect_text, line_col_to_byte, line_starts, options, parse_wikilink_url};
use super::embeds::EmbedRef;
use super::frontmatter::mask;
use super::md_links::MdLinkRef;
use super::tags::TagRef;
use super::wikilinks::WikiLinkRef;

/// One occurrence of a `[[]]`-family construct, or of a Markdown link that
/// points at a scrap file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WikiRef {
    /// Plain wikilink `[[name]]` / `[[a/b/name#h|alias]]`.
    Link(WikiLinkRef),
    /// Explicit tag `#[[tag]]` / `#[[a/b/c]]`.
    Tag(TagRef),
    /// Inline embed `![[name]]` / `![[name#heading]]`.
    Embed(EmbedRef),
    /// Markdown link to a scrap file `[text](/ctx/name.md#heading)`.
    MdLink(MdLinkRef),
}

/// Extract every `[[]]`-family occurrence from the markdown body in source
/// order. Comrak owns extraction of wiki-shaped syntax; scraps only classifies
/// the extracted node by the prefix immediately before `[[`.
pub fn wiki_refs(text: &str) -> Vec<WikiRef> {
    let arena = Arena::new();
    let opts = options();
    let parse_text = expose_embed_wikilinks(&mask(text));
    let root = parse_document(&arena, &parse_text, &opts);
    let starts = line_starts(text);

    root.descendants()
        .filter_map(|node| {
            if let NodeValue::Link(link) = &node.data().value {
                let NodeLink { url, .. } = link.as_ref();
                let line = node.data().sourcepos.start.line;
                return MdLinkRef::new(url, collect_text(node), line).map(WikiRef::MdLink);
            }
            let NodeValue::WikiLink(NodeWikiLink { url }) = &node.data().value else {
                return None;
            };
            if url.is_empty() {
                return None;
            }

            let pos = node.data().sourcepos;
            let line = pos.start.line;
            let byte = line_col_to_byte(&starts, line, pos.start.column);
            let prefix = byte.checked_sub(1).and_then(|i| text.as_bytes().get(i));
            let (ctx_path, title, heading) = parse_wikilink_url(url);
            let label = collect_text(node);
            let alias = (label != *url).then_some(label);

            match prefix {
                Some(b'#') => {
                    let mut path = ctx_path;
                    path.push(title);
                    if path.iter().all(|s| !s.is_empty()) {
                        Some(WikiRef::Tag(TagRef { path, line }))
                    } else {
                        None
                    }
                }
                Some(b'!') => {
                    if title.is_empty() || alias.is_some() {
                        None
                    } else {
                        Some(WikiRef::Embed(EmbedRef {
                            ctx_path,
                            title,
                            heading,
                            line,
                        }))
                    }
                }
                _ => Some(WikiRef::Link(WikiLinkRef {
                    ctx_path,
                    title,
                    heading,
                    alias,
                    line,
                })),
            }
        })
        .collect()
}

/// Comrak treats `![[x]]` as image-shaped Markdown, not as a WikiLink node.
/// Replacing only the attached `!` keeps byte positions stable while letting
/// comrak extract the same `[[x]]` shape that tags and links use.
fn expose_embed_wikilinks(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = bytes.to_vec();
    let mut i = 0;
    while i + 2 < bytes.len() {
        if bytes[i] == b'!' && bytes[i + 1] == b'[' && bytes[i + 2] == b'[' {
            out[i] = b' ';
        }
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::assert_matches;

    fn link(ctx_path: &[&str], title: &str) -> WikiLinkRef {
        WikiLinkRef {
            ctx_path: ctx_path.iter().map(|s| s.to_string()).collect(),
            title: title.to_string(),
            heading: None,
            alias: None,
            line: 1,
        }
    }

    #[rstest]
    #[case::basic("see [[scrap]]", "scrap")]
    #[case::japanese("see [[日本語]]", "日本語")]
    fn it_classifies_link_only(#[case] input: &str, #[case] title: &str) {
        assert_eq!(wiki_refs(input), vec![WikiRef::Link(link(&[], title))]);
    }

    #[rstest]
    #[case::basic("tagged #[[ai]]", &["ai"])]
    #[case::hierarchical("tagged #[[ai/ml]]", &["ai", "ml"])]
    #[case::japanese("tagged #[[プログラミング]]", &["プログラミング"])]
    fn it_classifies_tag_only(#[case] input: &str, #[case] path: &[&str]) {
        let expected: Vec<String> = path.iter().map(|s| s.to_string()).collect();
        let res = wiki_refs(input);
        assert_eq!(res.len(), 1);
        assert_matches!(&res[0], WikiRef::Tag(t) if t.path == expected);
    }

    #[rstest]
    #[case::basic("see ![[paper]]", "paper")]
    #[case::japanese("see ![[論文]]", "論文")]
    fn it_classifies_embed_only(#[case] input: &str, #[case] title: &str) {
        let res = wiki_refs(input);
        assert_eq!(res.len(), 1);
        assert_matches!(&res[0], WikiRef::Embed(e) if e.title == title);
    }

    #[rstest]
    #[case::same_paragraph("link [[scrap]] tag #[[ai]] embed ![[paper]]")]
    #[case::source_lines(
        "first line [[scrap]]\n\ntag #[[ai]] on line 3\n\nembed ![[paper]] on line 5"
    )]
    fn it_disambiguates_link_tag_embed(#[case] input: &str) {
        let res = wiki_refs(input);
        assert_eq!(res.len(), 3);
        assert_matches!(&res[0], WikiRef::Link(r) if r.title == "scrap");
        assert_matches!(&res[1], WikiRef::Tag(t) if t.path == vec!["ai".to_string()]);
        assert_matches!(&res[2], WikiRef::Embed(e) if e.title == "paper");
    }

    #[rstest]
    #[case::fenced_code("```\n[[code-link]]\n#[[code-tag]]\n![[code-embed]]\n```")]
    #[case::inline_code("`[[c-link]]` and `#[[c-tag]]` and `![[c-embed]]`")]
    #[case::unterminated("text #[[unterminated and ![[also without close")]
    #[case::empty("")]
    fn it_excludes_non_refs(#[case] input: &str) {
        let res = wiki_refs(input);
        assert!(res.is_empty());
    }

    fn md_link(dest: &str, alias: &str, line: usize) -> WikiRef {
        WikiRef::MdLink(MdLinkRef {
            dest: dest.to_string(),
            alias: Some(alias.to_string()),
            line,
        })
    }

    #[rstest]
    #[case::absolute(
        "[customers](/tables/customers.md)",
        "/tables/customers.md",
        "customers"
    )]
    #[case::relative("[other](./other.md)", "./other.md", "other")]
    #[case::fragment(
        "[r](../metrics/revenue.md#definition)",
        "../metrics/revenue.md#definition",
        "r"
    )]
    #[case::percent_encoded("[p](Static%20Site/Pipeline.md)", "Static%20Site/Pipeline.md", "p")]
    #[case::angle_brackets("[p](<Static Site/Pipeline.md>)", "Static Site/Pipeline.md", "p")]
    #[case::reference(
        "[customers][c]\n\n[c]: /tables/customers.md",
        "/tables/customers.md",
        "customers"
    )]
    #[case::formatted_text("[**bold** text](/a.md)", "/a.md", "bold text")]
    fn it_reads_a_markdown_link_to_a_scrap_file(
        #[case] input: &str,
        #[case] dest: &str,
        #[case] alias: &str,
    ) {
        assert_eq!(wiki_refs(input), vec![md_link(dest, alias, 1)]);
    }

    #[rstest]
    #[case::scheme("[a](https://example.com/a.md)")]
    #[case::mailto("[a](mailto:a@example.com)")]
    #[case::not_markdown("[a](/tables/customers.html)")]
    #[case::image("![a](/img/a.md)")]
    #[case::inline_code("`[a](/a.md)`")]
    #[case::fenced_code("```\n[a](/a.md)\n```")]
    #[case::frontmatter("---\nsee: \"[a](/a.md)\"\n---\n\nbody")]
    fn it_excludes_markdown_links_that_are_not_scrap_links(#[case] input: &str) {
        assert!(wiki_refs(input).is_empty(), "input was: {input:?}");
    }

    #[test]
    fn it_keeps_markdown_and_wiki_links_in_source_order() {
        let res = wiki_refs("[[a]] then [b](/b.md)\n\nthen [[c]] and [d](./d.md)");
        assert_eq!(res.len(), 4);
        assert_matches!(&res[0], WikiRef::Link(r) if r.title == "a");
        assert_eq!(res[1], md_link("/b.md", "b", 1));
        assert_matches!(&res[2], WikiRef::Link(r) if r.title == "c");
        assert_eq!(res[3], md_link("./d.md", "d", 3));
    }

    #[test]
    fn it_handles_unicode_inside_brackets() {
        let res = wiki_refs("[[日本語]] and #[[プログラミング]] and ![[論文]]");
        assert_eq!(res.len(), 3);
        assert_matches!(&res[0], WikiRef::Link(r) if r.title == "日本語");
        assert_matches!(&res[1], WikiRef::Tag(t) if t.path == vec!["プログラミング".to_string()]);
        assert_matches!(&res[2], WikiRef::Embed(e) if e.title == "論文");
    }

    #[test]
    fn it_reads_targets_in_nfc_and_keeps_alias_and_heading_as_written() {
        // `e` + U+0301 composes to U+00E9 in the target only.
        let res = wiki_refs(
            "[[cafe\u{301}/cafe\u{301}#cafe\u{301}|cafe\u{301}]] #[[cafe\u{301}]] ![[cafe\u{301}]] [[cafe\u{301}]]",
        );
        assert_eq!(res.len(), 4);
        let WikiRef::Link(r) = &res[0] else {
            panic!("expected Link");
        };
        assert_eq!(r.ctx_path, vec!["caf\u{e9}".to_string()]);
        assert_eq!(r.title, "caf\u{e9}");
        assert_eq!(r.heading.as_deref(), Some("cafe\u{301}"));
        assert_eq!(r.alias.as_deref(), Some("cafe\u{301}"));
        assert_matches!(&res[1], WikiRef::Tag(t) if t.path == vec!["caf\u{e9}".to_string()]);
        assert_matches!(&res[2], WikiRef::Embed(e) if e.title == "caf\u{e9}");
        assert_matches!(&res[3], WikiRef::Link(r) if r.title == "caf\u{e9}" && r.alias.is_none());
    }

    #[test]
    fn it_preserves_link_with_alias_and_heading() {
        let res = wiki_refs("[[Person/Eric Evans#bio|Eric]]");
        assert_eq!(res.len(), 1);
        let WikiRef::Link(r) = &res[0] else {
            panic!("expected Link");
        };
        assert_eq!(r.ctx_path, vec!["Person".to_string()]);
        assert_eq!(r.title, "Eric Evans");
        assert_eq!(r.heading.as_deref(), Some("bio"));
        assert_eq!(r.alias.as_deref(), Some("Eric"));
    }
}
