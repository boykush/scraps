//! Unified retrieval of wiki-shaped inline syntax: `[[link]]`, `#[[tag]]`,
//! and `![[embed]]` are all variants of the same `[[]]` family. This module
//! lets comrak extract the bracketed shape first, then classifies each
//! occurrence by the prefix immediately attached to the `[[`.

use comrak::{
    nodes::{NodeValue, NodeWikiLink},
    parse_document, Arena,
};

use super::common::{collect_text, line_col_to_byte, line_starts, options, parse_wikilink_url};
use super::embeds::EmbedRef;
use super::tags::TagRef;
use super::wikilinks::WikiLinkRef;

/// One occurrence of a `[[]]`-family construct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WikiRef {
    /// Plain wikilink `[[name]]` / `[[a/b/name#h|alias]]`.
    Link(WikiLinkRef),
    /// Explicit tag `#[[tag]]` / `#[[a/b/c]]`.
    Tag(TagRef),
    /// Inline embed `![[name]]` / `![[name#heading]]`.
    Embed(EmbedRef),
}

/// Extract every `[[]]`-family occurrence from the markdown body in source
/// order. Comrak owns extraction of wiki-shaped syntax; scraps only classifies
/// the extracted node by the prefix immediately before `[[`.
pub fn wiki_refs(text: &str) -> Vec<WikiRef> {
    let arena = Arena::new();
    let opts = options();
    let parse_text = expose_embed_wikilinks(text);
    let root = parse_document(&arena, &parse_text, &opts);
    let starts = line_starts(text);

    root.descendants()
        .filter_map(|node| {
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
            let display = match &heading {
                Some(h) => format!("{}#{}", url_path(&ctx_path, &title), h),
                None => url_path(&ctx_path, &title),
            };
            let alias = if label == display { None } else { Some(label) };

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
                })),
            }
        })
        .collect()
}

fn url_path(ctx_path: &[String], title: &str) -> String {
    if ctx_path.is_empty() {
        title.to_string()
    } else {
        format!("{}/{}", ctx_path.join("/"), title)
    }
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

    fn link(ctx_path: &[&str], title: &str) -> WikiLinkRef {
        WikiLinkRef {
            ctx_path: ctx_path.iter().map(|s| s.to_string()).collect(),
            title: title.to_string(),
            heading: None,
            alias: None,
        }
    }

    #[test]
    fn it_classifies_link_only() {
        let res = wiki_refs("see [[scrap]]");
        assert_eq!(res, vec![WikiRef::Link(link(&[], "scrap"))]);
    }

    #[test]
    fn it_classifies_tag_only() {
        let res = wiki_refs("tagged #[[ai]]");
        assert_eq!(res.len(), 1);
        assert!(matches!(&res[0], WikiRef::Tag(t) if t.path == vec!["ai".to_string()]));
    }

    #[test]
    fn it_classifies_embed_only() {
        let res = wiki_refs("see ![[paper]]");
        assert_eq!(res.len(), 1);
        assert!(matches!(&res[0], WikiRef::Embed(e) if e.title == "paper"));
    }

    #[test]
    fn it_disambiguates_link_tag_embed_in_one_paragraph() {
        let res = wiki_refs("link [[scrap]] tag #[[ai]] embed ![[paper]]");
        assert_eq!(res.len(), 3);
        assert!(matches!(&res[0], WikiRef::Link(r) if r.title == "scrap"));
        assert!(matches!(&res[1], WikiRef::Tag(t) if t.path == vec!["ai".to_string()]));
        assert!(matches!(&res[2], WikiRef::Embed(e) if e.title == "paper"));
    }

    #[test]
    fn it_does_not_double_count_tag_as_link() {
        // The smell this refactor addresses: `#[[ai]]` must be a Tag only,
        // never reported as a phantom Link "ai".
        let res = wiki_refs("only #[[ai]] here");
        assert_eq!(res.len(), 1);
        assert!(matches!(&res[0], WikiRef::Tag(_)));
    }

    #[test]
    fn it_does_not_double_count_embed_as_link() {
        let res = wiki_refs("only ![[paper]] here");
        assert_eq!(res.len(), 1);
        assert!(matches!(&res[0], WikiRef::Embed(_)));
    }

    #[test]
    fn it_orders_by_source_line() {
        let input = "first line [[a]]\n\ntag #[[t]] on line 3\n\nembed ![[e]] on line 5";
        let res = wiki_refs(input);
        assert_eq!(res.len(), 3);
        assert!(matches!(&res[0], WikiRef::Link(r) if r.title == "a"));
        assert!(matches!(&res[1], WikiRef::Tag(_)));
        assert!(matches!(&res[2], WikiRef::Embed(_)));
    }

    #[test]
    fn it_excludes_inside_code_block() {
        let input = "```\n[[code-link]]\n#[[code-tag]]\n![[code-embed]]\n```";
        let res = wiki_refs(input);
        assert!(res.is_empty());
    }

    #[test]
    fn it_excludes_inside_inline_code() {
        let input = "`[[c-link]]` and `#[[c-tag]]` and `![[c-embed]]`";
        let res = wiki_refs(input);
        assert!(res.is_empty());
    }

    #[test]
    fn it_handles_unicode_inside_brackets() {
        let res = wiki_refs("[[日本語]] and #[[プログラミング]] and ![[論文]]");
        assert_eq!(res.len(), 3);
        assert!(matches!(&res[0], WikiRef::Link(r) if r.title == "日本語"));
        assert!(matches!(&res[1], WikiRef::Tag(t) if t.path == vec!["プログラミング".to_string()]));
        assert!(matches!(&res[2], WikiRef::Embed(e) if e.title == "論文"));
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

    #[test]
    fn it_handles_unterminated_brackets_safely() {
        // Unterminated `#[[` or `![[` should not panic and should be left
        // alone (no Tag/Embed emitted, not masked).
        let res = wiki_refs("text #[[unterminated and ![[also without close");
        assert!(res.is_empty());
    }

    #[test]
    fn it_handles_empty_input() {
        let res = wiki_refs("");
        assert!(res.is_empty());
    }
}
