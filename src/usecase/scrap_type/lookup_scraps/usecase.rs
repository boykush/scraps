use crate::error::ScrapsResult;
use scraps_libs::model::context::Ctx;
use scraps_libs::model::scrap::Scrap;
use scraps_libs::model::title::Title;
use scraps_libs::nfc::nfc;

/// Result for type scraps lookup operation
#[derive(Debug, Clone, PartialEq)]
pub struct LookupTypeScrapsResult {
    pub title: Title,
    pub ctx: Option<Ctx>,
}

pub struct LookupTypeScrapsUsecase;

impl LookupTypeScrapsUsecase {
    pub fn new() -> LookupTypeScrapsUsecase {
        LookupTypeScrapsUsecase
    }

    /// The requested name is trimmed and read in NFC as a scrap's own type
    /// is, then compared exactly: a type nobody uses is an empty result, not an error.
    pub fn execute(
        &self,
        scraps: &[Scrap],
        type_name: &str,
    ) -> ScrapsResult<Vec<LookupTypeScrapsResult>> {
        let requested = nfc(type_name.trim());

        let mut results: Vec<LookupTypeScrapsResult> = scraps
            .iter()
            .filter(|scrap| scrap.scrap_type() == Some(requested.as_ref()))
            .map(|scrap| LookupTypeScrapsResult {
                title: scrap.title().clone(),
                ctx: scrap.ctx().clone(),
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

    fn titles(results: &[LookupTypeScrapsResult]) -> Vec<String> {
        results.iter().map(|r| r.title.to_string()).collect()
    }

    #[test]
    fn it_returns_only_the_scraps_of_the_type() {
        let scraps = vec![
            Scrap::new("post", &None, "---\ntype: Blog\n---\n"),
            Scrap::new("summary", &None, "---\ntype: Book\n---\n"),
            Scrap::new("note", &None, "# Note\n"),
        ];

        let results = LookupTypeScrapsUsecase::new()
            .execute(&scraps, "Blog")
            .unwrap();

        assert_eq!(titles(&results), vec!["post"]);
    }

    #[test]
    fn it_matches_a_type_written_in_another_normalization_form() {
        let scraps = vec![Scrap::new("post", &None, "---\ntype: caf\u{e9}\n---\n")];

        let results = LookupTypeScrapsUsecase::new()
            .execute(&scraps, "cafe\u{301}")
            .unwrap();

        assert_eq!(titles(&results), vec!["post"]);
    }

    #[test]
    fn it_sorts_by_ctx_then_title() {
        let scraps = vec![
            Scrap::new("z", &None, "---\ntype: Blog\n---\n"),
            Scrap::new("b", &Some(Ctx::from("Posts")), "---\ntype: Blog\n---\n"),
            Scrap::new("a", &Some(Ctx::from("Posts")), "---\ntype: Blog\n---\n"),
        ];

        let results = LookupTypeScrapsUsecase::new()
            .execute(&scraps, "Blog")
            .unwrap();

        assert_eq!(titles(&results), vec!["z", "a", "b"]);
        assert_eq!(results[1].ctx, Some(Ctx::from("Posts")));
    }

    #[test]
    fn it_compares_case_sensitively() {
        let scraps = vec![Scrap::new("post", &None, "---\ntype: Blog\n---\n")];

        let results = LookupTypeScrapsUsecase::new()
            .execute(&scraps, "blog")
            .unwrap();

        assert!(results.is_empty());
    }

    #[test]
    fn it_trims_the_requested_type() {
        let scraps = vec![Scrap::new("post", &None, "---\ntype: Blog\n---\n")];

        let results = LookupTypeScrapsUsecase::new()
            .execute(&scraps, " Blog ")
            .unwrap();

        assert_eq!(titles(&results), vec!["post"]);
    }

    #[test]
    fn it_does_not_read_a_tag_or_a_hub_link_as_a_type() {
        let scraps = vec![
            Scrap::new("post", &None, "# Post\n\n[[Blog]] #[[Blog]]\n"),
            Scrap::new("Blog", &None, "# Blog\n"),
        ];

        let results = LookupTypeScrapsUsecase::new()
            .execute(&scraps, "Blog")
            .unwrap();

        assert!(results.is_empty());
    }

    #[test]
    fn it_returns_nothing_for_a_blank_type() {
        let scraps = vec![Scrap::new("note", &None, "# Note\n")];

        let results = LookupTypeScrapsUsecase::new().execute(&scraps, "").unwrap();

        assert!(results.is_empty());
    }
}
