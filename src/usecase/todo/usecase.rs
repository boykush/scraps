use crate::error::ScrapsResult;
use scraps_libs::markdown::query::{task_items, TaskStatus};
use scraps_libs::model::context::Ctx;
use scraps_libs::model::scrap::Scrap;
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
    ) -> ScrapsResult<Vec<TodoResult>> {
        let mut results: Vec<TodoResult> = Vec::new();

        for scrap in scraps {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(s: &str) -> Ctx {
        Ctx::from(s)
    }

    #[test]
    fn it_aggregates_open_tasks_by_default() {
        let scraps = vec![
            Scrap::new("a", &None, "- [ ] one\n- [x] two\n"),
            Scrap::new("b", &None, "- [-] three\n- [ ] four\n"),
        ];

        let results = TodoUsecase::new()
            .execute(&scraps, StatusFilter::Open)
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
            .execute(&scraps, StatusFilter::Done)
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
            .execute(&scraps, StatusFilter::Deferred)
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
            .execute(&scraps, StatusFilter::All)
            .unwrap();

        assert_eq!(results.len(), 3);
    }

    #[test]
    fn it_records_line_number_and_ctx() {
        let scraps = vec![Scrap::new(
            "borrowing",
            &Some(ctx("programming/rust")),
            "# borrowing\n\n- [ ] implement Drop for X\n",
        )];

        let results = TodoUsecase::new()
            .execute(&scraps, StatusFilter::Open)
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
            .execute(&scraps, StatusFilter::Open)
            .unwrap();

        assert!(results.is_empty());
    }
}
