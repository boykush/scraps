use crate::error::ScrapsResult;
use scraps_libs::markdown::query::{task_items, TaskStatus};
use scraps_libs::model::context::Ctx;
use scraps_libs::model::scrap::Scrap;
use scraps_libs::model::tag::Tag;
use scraps_libs::model::title::Title;

/// Status filter for `scraps todo`. `All` short-circuits status filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusFilter {
    Open,
    Done,
    Deferred,
    All,
}

impl StatusFilter {
    fn matches(self, status: &TaskStatus) -> bool {
        match (self, status) {
            (StatusFilter::All, _) => true,
            (StatusFilter::Open, TaskStatus::Open) => true,
            (StatusFilter::Done, TaskStatus::Done) => true,
            (StatusFilter::Deferred, TaskStatus::Deferred) => true,
            _ => false,
        }
    }
}

/// One task list entry resolved back to its source scrap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoResult {
    pub title: Title,
    pub ctx: Option<Ctx>,
    pub status: TaskStatus,
    pub text: String,
    pub line: usize,
}

pub struct TodoUsecase;

impl TodoUsecase {
    pub fn new() -> TodoUsecase {
        TodoUsecase
    }

    pub fn execute(
        &self,
        scraps: &[Scrap],
        status_filter: StatusFilter,
        ctx_filter: Option<&Ctx>,
        tag_filter: Option<&Tag>,
    ) -> ScrapsResult<Vec<TodoResult>> {
        let mut results: Vec<TodoResult> = Vec::new();

        for scrap in scraps {
            if !scrap_matches_ctx(scrap, ctx_filter) {
                continue;
            }
            if !scrap_matches_tag(scrap, tag_filter) {
                continue;
            }

            for item in task_items(scrap.md_text()) {
                if !status_filter.matches(&item.status) {
                    continue;
                }
                results.push(TodoResult {
                    title: scrap.title().clone(),
                    ctx: scrap.ctx().clone(),
                    status: item.status,
                    text: item.text,
                    line: item.line,
                });
            }
        }

        results.sort_by(|a, b| {
            let a_ctx = a.ctx.as_ref().map(|c| c.to_string()).unwrap_or_default();
            let b_ctx = b.ctx.as_ref().map(|c| c.to_string()).unwrap_or_default();
            a_ctx
                .cmp(&b_ctx)
                .then_with(|| a.title.to_string().cmp(&b.title.to_string()))
                .then_with(|| a.line.cmp(&b.line))
        });

        Ok(results)
    }
}

/// Prefix-match on ctx segments. `--ctx programming` matches both
/// `programming` and `programming/rust`. `None` means "no filter" (matches
/// every scrap, including root scraps).
fn scrap_matches_ctx(scrap: &Scrap, filter: Option<&Ctx>) -> bool {
    let Some(filter) = filter else {
        return true;
    };
    let Some(scrap_ctx) = scrap.ctx() else {
        return false;
    };
    let filter_segs = filter.segments();
    let scrap_segs = scrap_ctx.segments();
    if filter_segs.len() > scrap_segs.len() {
        return false;
    }
    scrap_segs
        .iter()
        .zip(filter_segs.iter())
        .all(|(a, b)| a == b)
}

/// Tag matching uses Logseq-style auto-aggregation: `--tag a` matches scraps
/// tagged with `a`, `a/b`, or `a/b/c`. Mirrors `BacklinksMap::get_tag`.
fn scrap_matches_tag(scrap: &Scrap, filter: Option<&Tag>) -> bool {
    let Some(filter) = filter else {
        return true;
    };
    scrap
        .tags()
        .iter()
        .any(|tag| tag == filter || tag.ancestors().iter().any(|ancestor| ancestor == filter))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(s: &str) -> Ctx {
        Ctx::from(s)
    }

    fn tag(s: &str) -> Tag {
        Tag::from(s)
    }

    #[test]
    fn it_aggregates_open_tasks_by_default() {
        let scraps = vec![
            Scrap::new("a", &None, "- [ ] one\n- [x] two\n"),
            Scrap::new("b", &None, "- [-] three\n- [ ] four\n"),
        ];

        let results = TodoUsecase::new()
            .execute(&scraps, StatusFilter::Open, None, None)
            .unwrap();

        assert_eq!(results.len(), 2);
        let texts: Vec<&str> = results.iter().map(|r| r.text.as_str()).collect();
        assert!(texts.contains(&"one"));
        assert!(texts.contains(&"four"));
    }

    #[test]
    fn it_filters_by_status_done() {
        let scraps = vec![Scrap::new(
            "a",
            &None,
            "- [ ] open\n- [x] done\n- [-] deferred\n",
        )];

        let results = TodoUsecase::new()
            .execute(&scraps, StatusFilter::Done, None, None)
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, TaskStatus::Done);
        assert_eq!(results[0].text, "done");
    }

    #[test]
    fn it_filters_by_status_deferred() {
        let scraps = vec![Scrap::new(
            "a",
            &None,
            "- [ ] open\n- [x] done\n- [-] deferred\n",
        )];

        let results = TodoUsecase::new()
            .execute(&scraps, StatusFilter::Deferred, None, None)
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, TaskStatus::Deferred);
    }

    #[test]
    fn it_returns_all_statuses_when_filter_is_all() {
        let scraps = vec![Scrap::new(
            "a",
            &None,
            "- [ ] open\n- [x] done\n- [-] deferred\n",
        )];

        let results = TodoUsecase::new()
            .execute(&scraps, StatusFilter::All, None, None)
            .unwrap();

        assert_eq!(results.len(), 3);
    }

    #[test]
    fn it_filters_by_ctx_exact() {
        let scraps = vec![
            Scrap::new("borrowing", &Some(ctx("programming/rust")), "- [ ] one\n"),
            Scrap::new(
                "python-zen",
                &Some(ctx("programming/python")),
                "- [ ] two\n",
            ),
            Scrap::new("misc", &None, "- [ ] three\n"),
        ];

        let results = TodoUsecase::new()
            .execute(
                &scraps,
                StatusFilter::Open,
                Some(&ctx("programming/rust")),
                None,
            )
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "one");
    }

    #[test]
    fn it_filters_by_ctx_prefix() {
        // `--ctx programming` matches anything under programming/...
        let scraps = vec![
            Scrap::new("borrowing", &Some(ctx("programming/rust")), "- [ ] one\n"),
            Scrap::new(
                "python-zen",
                &Some(ctx("programming/python")),
                "- [ ] two\n",
            ),
            Scrap::new("design", &Some(ctx("misc")), "- [ ] three\n"),
        ];

        let results = TodoUsecase::new()
            .execute(&scraps, StatusFilter::Open, Some(&ctx("programming")), None)
            .unwrap();

        let texts: Vec<&str> = results.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(texts.len(), 2);
        assert!(texts.contains(&"one"));
        assert!(texts.contains(&"two"));
    }

    #[test]
    fn it_excludes_root_scraps_when_ctx_filter_set() {
        let scraps = vec![
            Scrap::new("root", &None, "- [ ] one\n"),
            Scrap::new("borrowing", &Some(ctx("programming")), "- [ ] two\n"),
        ];

        let results = TodoUsecase::new()
            .execute(&scraps, StatusFilter::Open, Some(&ctx("programming")), None)
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "two");
    }

    #[test]
    fn it_filters_by_tag_with_ancestor_aggregation() {
        let scraps = vec![
            Scrap::new("a", &None, "#[[programming/rust]]\n- [ ] one\n"),
            Scrap::new("b", &None, "#[[programming]]\n- [ ] two\n"),
            Scrap::new("c", &None, "#[[ai]]\n- [ ] three\n"),
        ];

        let results = TodoUsecase::new()
            .execute(&scraps, StatusFilter::Open, None, Some(&tag("programming")))
            .unwrap();

        let texts: Vec<&str> = results.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(texts.len(), 2);
        assert!(texts.contains(&"one"));
        assert!(texts.contains(&"two"));
    }

    #[test]
    fn it_combines_ctx_and_tag_filters() {
        let scraps = vec![
            Scrap::new(
                "borrowing",
                &Some(ctx("programming/rust")),
                "#[[memory]]\n- [ ] one\n",
            ),
            Scrap::new(
                "python-zen",
                &Some(ctx("programming/python")),
                "#[[memory]]\n- [ ] two\n",
            ),
            Scrap::new(
                "lifetimes",
                &Some(ctx("programming/rust")),
                "#[[ai]]\n- [ ] three\n",
            ),
        ];

        let results = TodoUsecase::new()
            .execute(
                &scraps,
                StatusFilter::Open,
                Some(&ctx("programming/rust")),
                Some(&tag("memory")),
            )
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "one");
    }

    #[test]
    fn it_records_line_number_and_ctx() {
        let scraps = vec![Scrap::new(
            "borrowing",
            &Some(ctx("programming/rust")),
            "# borrowing\n\n- [ ] implement Drop for X\n",
        )];

        let results = TodoUsecase::new()
            .execute(&scraps, StatusFilter::Open, None, None)
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "implement Drop for X");
        assert_eq!(results[0].line, 3);
        assert_eq!(results[0].title.to_string(), "borrowing");
        assert_eq!(
            results[0].ctx.as_ref().map(|c| c.to_string()),
            Some("programming/rust".to_string())
        );
    }

    #[test]
    fn it_returns_empty_when_no_scraps_have_tasks() {
        let scraps = vec![Scrap::new("a", &None, "# Just text, no tasks.\n")];

        let results = TodoUsecase::new()
            .execute(&scraps, StatusFilter::Open, None, None)
            .unwrap();

        assert!(results.is_empty());
    }
}
