//! Unified retrieval of wiki-shaped inline syntax: `[[link]]`, `#[[tag]]`,
//! and `![[embed]]` are all variants of the same `[[]]` family. This module
//! parses them in one pass and classifies each occurrence, so individual
//! consumers (wikilinks / tags / embeds) don't need to know about each other.

use comrak::{
    nodes::{NodeValue, NodeWikiLink},
    parse_document, Arena,
};

use super::common::{collect_text, options, parse_wikilink_url};
use super::embeds::{embeds, EmbedRef};
use super::tags::{tags, TagOccurrence};
use super::wikilinks::WikiLinkRef;

/// One occurrence of a `[[]]`-family construct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WikiLink {
    /// Plain wikilink `[[name]]` / `[[a/b/name#h|alias]]`.
    Link(WikiLinkRef),
    /// Explicit tag `#[[tag]]` / `#[[a/b/c]]`.
    Tag(TagOccurrence),
    /// Inline embed `![[name]]` / `![[name#heading]]`.
    Embed(EmbedRef),
}

/// Extract every `[[]]`-family occurrence from the markdown body in source
/// order. Tags and embeds are masked out before the link parser runs, so a
/// `#[[ai]]` is reported only as a `Tag` (never duplicated as a `Link`).
pub fn wiki_links(text: &str) -> Vec<WikiLink> {
    let tag_occs = tags(text);
    let embed_refs = embeds(text);

    // Mask out `#[[…]]` and `![[…]]` byte ranges so the wikilink parser
    // (comrak) doesn't pick those up as plain `[[…]]` again.
    let masked = mask_tag_and_embed_syntax(text);

    let arena = Arena::new();
    let opts = options();
    let root = parse_document(&arena, &masked, &opts);
    let link_pairs: Vec<(usize, WikiLinkRef)> = root
        .descendants()
        .filter_map(|node| {
            let NodeValue::WikiLink(NodeWikiLink { url }) = &node.data().value else {
                return None;
            };
            if url.is_empty() {
                return None;
            }
            let line = node.data().sourcepos.start.line;
            let (ctx_path, title, heading) = parse_wikilink_url(url);
            let label = collect_text(node);
            let display = match &heading {
                Some(h) => format!("{}#{}", url_path(&ctx_path, &title), h),
                None => url_path(&ctx_path, &title),
            };
            let alias = if label == display { None } else { Some(label) };
            Some((
                line,
                WikiLinkRef {
                    ctx_path,
                    title,
                    heading,
                    alias,
                },
            ))
        })
        .collect();

    // Combine and sort by source line for stable order.
    let mut all: Vec<(usize, WikiLink)> = Vec::new();
    for (line, r) in link_pairs {
        all.push((line, WikiLink::Link(r)));
    }
    for occ in tag_occs {
        all.push((occ.line, WikiLink::Tag(occ)));
    }
    for embed in embed_refs {
        all.push((embed.line, WikiLink::Embed(embed)));
    }
    all.sort_by_key(|(line, _)| *line);
    all.into_iter().map(|(_, wl)| wl).collect()
}

fn url_path(ctx_path: &[String], title: &str) -> String {
    if ctx_path.is_empty() {
        title.to_string()
    } else {
        format!("{}/{}", ctx_path.join("/"), title)
    }
}

/// Replace every byte of `#[[…]]` and `![[…]]` (including the closing `]]`)
/// with ASCII spaces. Length is preserved so line and column offsets remain
/// valid for downstream parsing.
fn mask_tag_and_embed_syntax(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out: Vec<u8> = bytes.to_vec();
    let mut i = 0;
    while i + 4 < bytes.len() {
        let prefix_match =
            (bytes[i] == b'#' || bytes[i] == b'!') && bytes[i + 1] == b'[' && bytes[i + 2] == b'[';
        if !prefix_match {
            i += 1;
            continue;
        }
        // Search for the closing `]]` on the same line.
        let mut j = i + 3;
        let mut found = false;
        while j + 1 < bytes.len() {
            if bytes[j] == b'\n' {
                break;
            }
            if bytes[j] == b']' && bytes[j + 1] == b']' {
                found = true;
                break;
            }
            j += 1;
        }
        if found {
            for k in i..(j + 2) {
                out[k] = b' ';
            }
            i = j + 2;
        } else {
            i += 1;
        }
    }
    // The original text was valid UTF-8 and we only replaced complete byte
    // ranges with ASCII spaces, so the result is also valid UTF-8.
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
        let res = wiki_links("see [[scrap]]");
        assert_eq!(res, vec![WikiLink::Link(link(&[], "scrap"))]);
    }

    #[test]
    fn it_classifies_tag_only() {
        let res = wiki_links("tagged #[[ai]]");
        assert_eq!(res.len(), 1);
        assert!(matches!(&res[0], WikiLink::Tag(t) if t.path == vec!["ai".to_string()]));
    }

    #[test]
    fn it_classifies_embed_only() {
        let res = wiki_links("see ![[paper]]");
        assert_eq!(res.len(), 1);
        assert!(matches!(&res[0], WikiLink::Embed(e) if e.title == "paper"));
    }

    #[test]
    fn it_disambiguates_link_tag_embed_in_one_paragraph() {
        let res = wiki_links("link [[scrap]] tag #[[ai]] embed ![[paper]]");
        assert_eq!(res.len(), 3);
        assert!(matches!(&res[0], WikiLink::Link(r) if r.title == "scrap"));
        assert!(matches!(&res[1], WikiLink::Tag(t) if t.path == vec!["ai".to_string()]));
        assert!(matches!(&res[2], WikiLink::Embed(e) if e.title == "paper"));
    }

    #[test]
    fn it_does_not_double_count_tag_as_link() {
        // The smell this refactor addresses: `#[[ai]]` must be a Tag only,
        // never reported as a phantom Link "ai".
        let res = wiki_links("only #[[ai]] here");
        assert_eq!(res.len(), 1);
        assert!(matches!(&res[0], WikiLink::Tag(_)));
    }

    #[test]
    fn it_does_not_double_count_embed_as_link() {
        let res = wiki_links("only ![[paper]] here");
        assert_eq!(res.len(), 1);
        assert!(matches!(&res[0], WikiLink::Embed(_)));
    }

    #[test]
    fn it_orders_by_source_line() {
        let input = "first line [[a]]\n\ntag #[[t]] on line 3\n\nembed ![[e]] on line 5";
        let res = wiki_links(input);
        assert_eq!(res.len(), 3);
        assert!(matches!(&res[0], WikiLink::Link(r) if r.title == "a"));
        assert!(matches!(&res[1], WikiLink::Tag(_)));
        assert!(matches!(&res[2], WikiLink::Embed(_)));
    }

    #[test]
    fn it_excludes_inside_code_block() {
        let input = "```\n[[code-link]]\n#[[code-tag]]\n![[code-embed]]\n```";
        let res = wiki_links(input);
        assert!(res.is_empty());
    }

    #[test]
    fn it_excludes_inside_inline_code() {
        let input = "`[[c-link]]` and `#[[c-tag]]` and `![[c-embed]]`";
        let res = wiki_links(input);
        assert!(res.is_empty());
    }

    #[test]
    fn it_handles_unicode_inside_brackets() {
        let res = wiki_links("[[日本語]] and #[[プログラミング]] and ![[論文]]");
        assert_eq!(res.len(), 3);
        assert!(matches!(&res[0], WikiLink::Link(r) if r.title == "日本語"));
        assert!(
            matches!(&res[1], WikiLink::Tag(t) if t.path == vec!["プログラミング".to_string()])
        );
        assert!(matches!(&res[2], WikiLink::Embed(e) if e.title == "論文"));
    }

    #[test]
    fn it_preserves_link_with_alias_and_heading() {
        let res = wiki_links("[[Person/Eric Evans#bio|Eric]]");
        assert_eq!(res.len(), 1);
        let WikiLink::Link(r) = &res[0] else {
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
        let res = wiki_links("text #[[unterminated and ![[also without close");
        assert!(res.is_empty());
    }

    #[test]
    fn it_handles_empty_input() {
        let res = wiki_links("");
        assert!(res.is_empty());
    }
}
