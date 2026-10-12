use std::collections::BTreeMap;

use crate::error::ScrapsResult;
use scraps_libs::model::scrap::Scrap;

/// One type in use, with the number of scraps that carry it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListTypeResult {
    pub name: String,
    pub scraps_count: usize,
}

pub struct ListTypeUsecase;

impl ListTypeUsecase {
    pub fn new() -> ListTypeUsecase {
        ListTypeUsecase
    }

    /// Types are whatever strings the scraps carry: there is no registry, so
    /// `Blog` and `blog` are two types. Most used first, then by name.
    pub fn execute(&self, scraps: &[Scrap]) -> ScrapsResult<Vec<ListTypeResult>> {
        let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
        for name in scraps.iter().filter_map(Scrap::scrap_type) {
            *counts.entry(name).or_default() += 1;
        }

        let mut results: Vec<ListTypeResult> = counts
            .into_iter()
            .map(|(name, scraps_count)| ListTypeResult {
                name: name.to_string(),
                scraps_count,
            })
            .collect();
        results.sort_by_key(|result| std::cmp::Reverse(result.scraps_count));

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result(name: &str, scraps_count: usize) -> ListTypeResult {
        ListTypeResult {
            name: name.to_string(),
            scraps_count,
        }
    }

    #[test]
    fn it_counts_the_scraps_of_each_type() {
        let scraps = vec![
            Scrap::new("a", &None, "---\ntype: Blog\n---\n"),
            Scrap::new("b", &None, "---\ntype: Book\n---\n"),
            Scrap::new("c", &None, "---\ntype: Book\n---\n"),
        ];

        let results = ListTypeUsecase::new().execute(&scraps).unwrap();

        assert_eq!(results, vec![result("Book", 2), result("Blog", 1)]);
    }

    #[test]
    fn it_orders_equally_used_types_by_name() {
        let scraps = vec![
            Scrap::new("a", &None, "---\ntype: Playbook\n---\n"),
            Scrap::new("b", &None, "---\ntype: Metric\n---\n"),
        ];

        let results = ListTypeUsecase::new().execute(&scraps).unwrap();

        assert_eq!(results, vec![result("Metric", 1), result("Playbook", 1)]);
    }

    #[test]
    fn it_leaves_untyped_scraps_out() {
        let scraps = vec![
            Scrap::new("a", &None, "# A\n\n#[[Blog]]\n"),
            Scrap::new("b", &None, "---\nstatus: draft\n---\n"),
            Scrap::new("c", &None, "---\ntype: [Blog]\n---\n"),
        ];

        assert!(ListTypeUsecase::new().execute(&scraps).unwrap().is_empty());
    }

    #[test]
    fn it_keeps_types_that_differ_only_in_case_apart() {
        let scraps = vec![
            Scrap::new("a", &None, "---\ntype: Blog\n---\n"),
            Scrap::new("b", &None, "---\ntype: blog\n---\n"),
        ];

        let results = ListTypeUsecase::new().execute(&scraps).unwrap();

        assert_eq!(results, vec![result("Blog", 1), result("blog", 1)]);
    }
}
