use crate::error::ScrapsResult;
use scraps_libs::markdown::query::{wiki_refs, WikiRef};
use scraps_libs::model::context::Ctx;
use scraps_libs::model::key::ScrapKey;
use scraps_libs::model::scrap::Scrap;
use scraps_libs::model::title::Title;
use std::collections::HashMap;

/// Kind of an outbound reference occurrence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkRefKind {
    /// Plain wikilink `[[...]]`.
    Link,
    /// Inline embed `![[...]]`.
    Embed,
}

/// One outbound reference occurrence found in a scrap.
#[derive(Debug, Clone, PartialEq)]
pub struct LookupScrapLinksResult {
    pub kind: LinkRefKind,
    pub title: Title,
    pub ctx: Option<Ctx>,
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
        title: &Title,
        ctx: &Option<Ctx>,
    ) -> ScrapsResult<Vec<LookupScrapLinksResult>> {
        let target_key = if let Some(ctx) = ctx {
            ScrapKey::with_ctx(title, ctx)
        } else {
            ScrapKey::from(title.clone())
        };

        let target_scrap = scraps
            .iter()
            .find(|scrap| scrap.self_key() == target_key)
            .ok_or_else(|| {
                anyhow::anyhow!("Scrap not found: title='{}', ctx='{:?}'", title, ctx)
            })?;

        let scrap_map: HashMap<ScrapKey, &Scrap> = scraps
            .iter()
            .map(|scrap| (scrap.self_key(), scrap))
            .collect();

        let results: Vec<LookupScrapLinksResult> = wiki_refs(target_scrap.md_text())
            .into_iter()
            .filter_map(|wref| match wref {
                WikiRef::Link(r) => {
                    let key = ScrapKey::from_path_str(&join_path(&r.ctx_path, &r.title));
                    scrap_map.get(&key).map(|linked| {
                        let linked_key = linked.self_key();
                        LookupScrapLinksResult {
                            kind: LinkRefKind::Link,
                            title: (&linked_key).into(),
                            ctx: (&linked_key).into(),
                            heading: r.heading,
                        }
                    })
                }
                WikiRef::Embed(r) => {
                    let key = ScrapKey::from_path_str(&join_path(&r.ctx_path, &r.title));
                    scrap_map.get(&key).map(|linked| {
                        let linked_key = linked.self_key();
                        LookupScrapLinksResult {
                            kind: LinkRefKind::Embed,
                            title: (&linked_key).into(),
                            ctx: (&linked_key).into(),
                            heading: r.heading,
                        }
                    })
                }
                WikiRef::Tag(_) => None,
            })
            .collect();

        Ok(results)
    }
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
            .execute(&scraps, &Title::from("scrap1"), &None)
            .expect("Should succeed");

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].kind, LinkRefKind::Link);
        assert_eq!(results[0].title.to_string(), "scrap2");
        assert_eq!(results[1].kind, LinkRefKind::Link);
        assert_eq!(results[1].title.to_string(), "scrap3");
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
            .execute(&scraps, &Title::from("scrap1"), &Some(Ctx::from("Context")))
            .expect("Should succeed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title.to_string(), "scrap2");
    }

    #[test]
    fn test_lookup_scrap_links_not_found() {
        let scraps = vec![Scrap::new("scrap1", &None, "# Scrap 1\n\nContent.")];

        let usecase = LookupScrapLinksUsecase::new();

        let result = usecase.execute(&scraps, &Title::from("Nonexistent Scrap"), &None);

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
            .execute(&scraps, &Title::from("scrap1"), &None)
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
            .execute(&scraps, &Title::from("scrap1"), &None)
            .expect("Should succeed");

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].kind, LinkRefKind::Link);
        assert_eq!(results[0].title.to_string(), "Target");
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
            .execute(&scraps, &Title::from("scrap1"), &None)
            .expect("Should succeed");

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].kind, LinkRefKind::Embed);
        assert_eq!(results[0].title.to_string(), "Target");
        assert_eq!(results[0].heading.as_deref(), Some("Install"));
        assert_eq!(results[1].kind, LinkRefKind::Link);
        assert_eq!(results[1].title.to_string(), "Other");
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
            .execute(&scraps, &Title::from("scrap1"), &None)
            .expect("Should succeed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title.to_string(), "Target");
    }

    #[test]
    fn test_lookup_scrap_links_resolves_ctx_target() {
        let scraps = vec![
            Scrap::new("scrap1", &None, "# Scrap 1\n\nSee [[Book/Target]]."),
            Scrap::new("Target", &Some("Book".into()), "# Target"),
        ];

        let usecase = LookupScrapLinksUsecase::new();
        let results = usecase
            .execute(&scraps, &Title::from("scrap1"), &None)
            .expect("Should succeed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title.to_string(), "Target");
        assert_eq!(
            results[0].ctx.as_ref().map(|c| c.to_string()).as_deref(),
            Some("Book")
        );
    }
}
