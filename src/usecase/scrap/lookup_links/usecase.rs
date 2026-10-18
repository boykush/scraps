use crate::error::ScrapsResult;
use crate::usecase::scrap::scrap_ref::{ScrapRef, find_scrap};
use scraps_libs::markdown::query::WikiRef;
use scraps_libs::model::key::ScrapKey;
use scraps_libs::model::scrap::Scrap;
use std::collections::HashMap;

/// Kind of an outbound reference occurrence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkRefKind {
    /// Plain wikilink `[[...]]`, or a Markdown link to a scrap file.
    Link,
    /// Inline embed `![[...]]`.
    Embed,
    /// A frontmatter `sources` entry whose `resource` is a path to a scrap.
    Source,
}

/// One outbound reference occurrence found in a scrap.
#[derive(Debug, Clone, PartialEq)]
pub struct LookupScrapLinksResult {
    pub kind: LinkRefKind,
    pub scrap: ScrapRef,
    pub heading: Option<String>,
}

pub struct LookupScrapLinksUsecase;

impl LookupScrapLinksUsecase {
    pub fn new() -> LookupScrapLinksUsecase {
        LookupScrapLinksUsecase
    }

    pub fn execute(
        &self,
        scraps: &[Scrap],
        key: &ScrapKey,
    ) -> ScrapsResult<Vec<LookupScrapLinksResult>> {
        let target_scrap = find_scrap(scraps, key)?;

        let scrap_map: HashMap<ScrapKey, &Scrap> = scraps
            .iter()
            .map(|scrap| (scrap.self_key(), scrap))
            .collect();

        let results: Vec<LookupScrapLinksResult> = target_scrap
            .refs()
            .iter()
            .filter_map(|wref| match wref {
                WikiRef::Link(r) => {
                    let key = ScrapKey::from_path_str(&join_path(&r.ctx_path, &r.title));
                    link_result(&scrap_map, &key, r.heading.clone())
                }
                WikiRef::MdLink(r) => {
                    let key = r.resolve(target_scrap.ctx())?;
                    link_result(&scrap_map, &key, r.heading())
                }
                WikiRef::Embed(r) => {
                    let key = ScrapKey::from_path_str(&join_path(&r.ctx_path, &r.title));
                    scrap_map.get(&key).map(|linked| LookupScrapLinksResult {
                        kind: LinkRefKind::Embed,
                        scrap: (*linked).into(),
                        heading: r.heading.clone(),
                    })
                }
                WikiRef::Tag(_) => None,
            })
            .chain(target_scrap.source_links().into_iter().filter_map(|link| {
                let linked = scrap_map.get(&link.target?)?;
                Some(LookupScrapLinksResult {
                    kind: LinkRefKind::Source,
                    scrap: (*linked).into(),
                    heading: link.heading,
                })
            }))
            .collect();

        Ok(results)
    }
}

fn link_result(
    scrap_map: &HashMap<ScrapKey, &Scrap>,
    key: &ScrapKey,
    heading: Option<String>,
) -> Option<LookupScrapLinksResult> {
    scrap_map.get(key).map(|linked| LookupScrapLinksResult {
        kind: LinkRefKind::Link,
        scrap: (*linked).into(),
        heading,
    })
}

fn join_path(ctx_path: &[String], title: &str) -> String {
    if ctx_path.is_empty() {
        title.to_string()
    } else {
        format!("{}/{}", ctx_path.join("/"), title)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraps_libs::model::context::Ctx;

    #[test]
    fn test_lookup_scrap_links_success() {
        let scraps = vec![
            Scrap::new(
                "scrap1",
                &None,
                "# Scrap 1\n\nThis links to [[scrap2]] and [[scrap3]].",
            ),
            Scrap::new("scrap2", &None, "# Scrap 2\n\nContent of scrap 2."),
            Scrap::new("scrap3", &None, "# Scrap 3\n\nContent of scrap 3."),
        ];

        let usecase = LookupScrapLinksUsecase::new();

        let results = usecase
            .execute(&scraps, &ScrapKey::from_path_str("scrap1"))
            .expect("Should succeed");

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].kind, LinkRefKind::Link);
        assert_eq!(results[0].scrap.title, "scrap2");
        assert_eq!(results[1].kind, LinkRefKind::Link);
        assert_eq!(results[1].scrap.title, "scrap3");
    }

    #[test]
    fn test_lookup_scrap_links_with_context() {
        let scraps = vec![
            Scrap::new(
                "scrap1",
                &Some("Context".into()),
                "# Scrap 1\n\nThis links to [[scrap2]].",
            ),
            Scrap::new("scrap2", &None, "# Scrap 2\n\nContent of scrap 2."),
        ];

        let usecase = LookupScrapLinksUsecase::new();

        let results = usecase
            .execute(&scraps, &ScrapKey::from_path_str("Context/scrap1"))
            .expect("Should succeed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].scrap.title, "scrap2");
    }

    #[test]
    fn test_lookup_scrap_links_follows_markdown_links() {
        let scraps = vec![
            Scrap::new(
                "orders",
                &Some("tables".into()),
                "[c](/tables/customers.md) [r](../metrics/revenue.md#definition) [g](./gone.md)",
            ),
            Scrap::new("customers", &Some("tables".into()), ""),
            Scrap::new("revenue", &Some("metrics".into()), "# Definition"),
        ];

        let results = LookupScrapLinksUsecase::new()
            .execute(&scraps, &ScrapKey::from_path_str("tables/orders"))
            .expect("Should succeed");

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].kind, LinkRefKind::Link);
        assert_eq!(results[0].scrap.title, "customers");
        assert_eq!(results[0].scrap.ctx(), &Some(Ctx::from("tables")));
        assert_eq!(results[1].scrap.title, "revenue");
        assert_eq!(results[1].heading.as_deref(), Some("definition"));
    }

    #[test]
    fn test_lookup_scrap_links_not_found() {
        let scraps = vec![Scrap::new("scrap1", &None, "# Scrap 1\n\nContent.")];

        let usecase = LookupScrapLinksUsecase::new();

        let result = usecase.execute(&scraps, &ScrapKey::from_path_str("Nonexistent Scrap"));

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Scrap not found"));
    }

    #[test]
    fn test_lookup_scrap_links_no_links() {
        let scraps = vec![Scrap::new(
            "scrap1",
            &None,
            "# Scrap 1\n\nThis scrap has no links.",
        )];

        let usecase = LookupScrapLinksUsecase::new();

        let results = usecase
            .execute(&scraps, &ScrapKey::from_path_str("scrap1"))
            .expect("Should succeed");

        assert_eq!(results.len(), 0);
    }

    // Heading-qualified references for the same target are preserved as
    // separate occurrences so callers can distinguish them.
    #[test]
    fn test_lookup_scrap_links_preserves_heading_occurrences() {
        let scraps = vec![
            Scrap::new(
                "scrap1",
                &None,
                "# Scrap 1\n\nSee [[Target#Install]] and [[Target#Usage]].",
            ),
            Scrap::new("Target", &None, "# Target"),
        ];

        let usecase = LookupScrapLinksUsecase::new();
        let results = usecase
            .execute(&scraps, &ScrapKey::from_path_str("scrap1"))
            .expect("Should succeed");

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].kind, LinkRefKind::Link);
        assert_eq!(results[0].scrap.title, "Target");
        assert_eq!(results[0].heading.as_deref(), Some("Install"));
        assert_eq!(results[1].heading.as_deref(), Some("Usage"));
    }

    #[test]
    fn test_lookup_scrap_links_includes_embeds() {
        let scraps = vec![
            Scrap::new(
                "scrap1",
                &None,
                "# Scrap 1\n\nEmbed ![[Target#Install]] and link [[Other]].",
            ),
            Scrap::new("Target", &None, "# Target"),
            Scrap::new("Other", &None, "# Other"),
        ];

        let usecase = LookupScrapLinksUsecase::new();
        let results = usecase
            .execute(&scraps, &ScrapKey::from_path_str("scrap1"))
            .expect("Should succeed");

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].kind, LinkRefKind::Embed);
        assert_eq!(results[0].scrap.title, "Target");
        assert_eq!(results[0].heading.as_deref(), Some("Install"));
        assert_eq!(results[1].kind, LinkRefKind::Link);
        assert_eq!(results[1].scrap.title, "Other");
        assert_eq!(results[1].heading, None);
    }

    #[test]
    fn test_lookup_scrap_links_excludes_tags() {
        let scraps = vec![
            Scrap::new(
                "scrap1",
                &None,
                "# Scrap 1\n\nLink [[Target]] and tag #[[ai]].",
            ),
            Scrap::new("Target", &None, "# Target"),
        ];

        let usecase = LookupScrapLinksUsecase::new();
        let results = usecase
            .execute(&scraps, &ScrapKey::from_path_str("scrap1"))
            .expect("Should succeed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].scrap.title, "Target");
    }

    #[test]
    fn test_lookup_scrap_links_resolves_ctx_target() {
        let scraps = vec![
            Scrap::new("scrap1", &None, "# Scrap 1\n\nSee [[Book/Target]]."),
            Scrap::new("Target", &Some("Book".into()), "# Target"),
        ];

        let usecase = LookupScrapLinksUsecase::new();
        let results = usecase
            .execute(&scraps, &ScrapKey::from_path_str("scrap1"))
            .expect("Should succeed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].scrap.title, "Target");
        assert_eq!(
            results[0]
                .scrap
                .ctx()
                .as_ref()
                .map(|c| c.to_string())
                .as_deref(),
            Some("Book")
        );
    }

    #[test]
    fn test_lookup_scrap_links_lists_sources_naming_a_scrap_after_the_body() {
        let scraps = vec![
            Scrap::new(
                "okf",
                &None,
                "---\nsources:\n  - resource: https://example.com\n  - resource: /Blog/Intro.md\n  - resource: /Gone.md\n---\n\n[[Intro]]\n",
            ),
            Scrap::new("Intro", &Some(Ctx::from("Blog")), ""),
            Scrap::new("Intro", &None, ""),
        ];

        let results = LookupScrapLinksUsecase::new()
            .execute(&scraps, &ScrapKey::from_path_str("okf"))
            .unwrap();

        let kinds: Vec<LinkRefKind> = results.iter().map(|r| r.kind).collect();
        assert_eq!(kinds, vec![LinkRefKind::Link, LinkRefKind::Source]);
        assert_eq!(results[1].scrap.key, ScrapKey::from_path_str("Blog/Intro"));
    }
}
