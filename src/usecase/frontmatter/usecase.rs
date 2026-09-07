use crate::error::ScrapsResult;
use scraps_libs::markdown::query::frontmatter;
use scraps_libs::model::context::Ctx;
use scraps_libs::model::scrap::Scrap;
use scraps_libs::model::title::Title;
use serde_json::Value;

/// One scrap's frontmatter, resolved back to the scrap it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontmatterResult {
    pub title: Title,
    pub ctx: Option<Ctx>,
    pub frontmatter: Value,
}

pub struct FrontmatterUsecase;

impl FrontmatterUsecase {
    pub fn new() -> FrontmatterUsecase {
        FrontmatterUsecase
    }

    pub fn execute(&self, scraps: &[Scrap]) -> ScrapsResult<Vec<FrontmatterResult>> {
        let mut results: Vec<FrontmatterResult> = scraps
            .iter()
            .filter_map(|scrap| {
                frontmatter(scrap.md_text()).map(|value| FrontmatterResult {
                    title: scrap.title().clone(),
                    ctx: scrap.ctx().clone(),
                    frontmatter: value,
                })
            })
            .collect();

        results.sort_by(|a, b| {
            let a_ctx = a.ctx.as_ref().map(|c| c.to_string()).unwrap_or_default();
            let b_ctx = b.ctx.as_ref().map(|c| c.to_string()).unwrap_or_default();
            a_ctx
                .cmp(&b_ctx)
                .then_with(|| a.title.to_string().cmp(&b.title.to_string()))
        });

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ctx(s: &str) -> Ctx {
        Ctx::from(s)
    }

    #[test]
    fn it_collects_frontmatter_from_scraps_that_have_it() {
        let scraps = vec![
            Scrap::new("a", &None, "---\nstatus: draft\n---\n\n# A\n"),
            Scrap::new("b", &None, "# B\n\nNo frontmatter.\n"),
        ];

        let results = FrontmatterUsecase::new().execute(&scraps).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title.to_string(), "a");
        assert_eq!(results[0].frontmatter, json!({"status": "draft"}));
    }

    #[test]
    fn it_passes_keys_through_without_interpreting_them() {
        let scraps = vec![Scrap::new(
            "a",
            &None,
            "---\nowner: alice\ntags:\n  - infra\nnested:\n  k: v\n---\n",
        )];

        let results = FrontmatterUsecase::new().execute(&scraps).unwrap();

        assert_eq!(
            results[0].frontmatter,
            json!({"owner": "alice", "tags": ["infra"], "nested": {"k": "v"}})
        );
    }

    #[test]
    fn it_sorts_by_ctx_then_title() {
        let scraps = vec![
            Scrap::new("z", &None, "---\nk: 1\n---\n"),
            Scrap::new("b", &Some(ctx("Book")), "---\nk: 2\n---\n"),
            Scrap::new("a", &Some(ctx("Book")), "---\nk: 3\n---\n"),
        ];

        let results = FrontmatterUsecase::new().execute(&scraps).unwrap();

        let order: Vec<String> = results.iter().map(|r| r.title.to_string()).collect();
        assert_eq!(order, vec!["z", "a", "b"]);
    }

    #[test]
    fn it_skips_malformed_frontmatter_rather_than_failing() {
        let scraps = vec![Scrap::new(
            "a",
            &None,
            "---\nstatus: [unclosed\n---\n\n# A\n",
        )];

        let results = FrontmatterUsecase::new().execute(&scraps).unwrap();

        assert!(results.is_empty());
    }

    #[test]
    fn it_returns_empty_when_no_scrap_has_frontmatter() {
        let scraps = vec![Scrap::new("a", &None, "# A\n\nJust prose.\n")];

        assert!(FrontmatterUsecase::new()
            .execute(&scraps)
            .unwrap()
            .is_empty());
    }
}
