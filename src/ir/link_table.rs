use std::collections::HashSet;

use scraps_libs::markdown::query::WikiRef;
use scraps_libs::model::{key::ScrapKey, scrap::Scrap};

use super::schema::{EdgeKind, KeyJson, LinkEdge};

/// Resolve every link and embed occurrence against the scraps that exist.
/// This is the link step: the objects say what each scrap references, the
/// table says whether each reference lands.
pub fn link_edges(scraps: &[Scrap]) -> Vec<LinkEdge> {
    let keys: HashSet<ScrapKey> = scraps.iter().map(Scrap::self_key).collect();
    let keys = &keys;

    scraps
        .iter()
        .flat_map(|scrap| {
            let from = KeyJson::from(&scrap.self_key());
            scrap.refs().iter().filter_map(move |r| {
                let (to, kind, heading, line) = match r {
                    WikiRef::Link(l) => (ScrapKey::from(l), EdgeKind::Link, &l.heading, l.line),
                    WikiRef::Embed(e) => (
                        ScrapKey::from_path_str(&embed_path(&e.ctx_path, &e.title)),
                        EdgeKind::Embed,
                        &e.heading,
                        e.line,
                    ),
                    WikiRef::Tag(_) => return None,
                };
                Some(LinkEdge {
                    from: from.clone(),
                    resolved: keys.contains(&to),
                    to: KeyJson::from(&to),
                    kind,
                    heading: heading.clone(),
                    line,
                })
            })
        })
        .collect()
}

fn embed_path(ctx_path: &[String], title: &str) -> String {
    if ctx_path.is_empty() {
        title.to_string()
    } else {
        format!("{}/{}", ctx_path.join("/"), title)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_resolves_links_and_embeds_and_skips_tags() {
        let scraps = vec![
            Scrap::new(
                "a",
                &None,
                "line one\n[[b#Sec]] #[[tag]]\n![[Ctx/c]] [[missing]]",
            ),
            Scrap::new("b", &None, "## Sec"),
            Scrap::new("c", &Some("Ctx".into()), ""),
        ];

        let edges = link_edges(&scraps);

        assert_eq!(edges.len(), 3);
        assert_eq!(edges[0].to.title, "b");
        assert_eq!(edges[0].kind, EdgeKind::Link);
        assert_eq!(edges[0].heading.as_deref(), Some("Sec"));
        assert_eq!(edges[0].line, 2);
        assert!(edges[0].resolved);
        assert_eq!(edges[1].kind, EdgeKind::Embed);
        assert_eq!(edges[1].to.ctx.as_deref(), Some("Ctx"));
        assert!(edges[1].resolved);
        assert_eq!(edges[2].to.title, "missing");
        assert!(!edges[2].resolved);
        assert!(edges.iter().all(|e| e.from.title == "a"));
    }

    #[test]
    fn it_keeps_duplicate_occurrences() {
        let scraps = vec![Scrap::new("a", &None, "[[a]] [[a]]")];
        assert_eq!(link_edges(&scraps).len(), 2);
    }
}
