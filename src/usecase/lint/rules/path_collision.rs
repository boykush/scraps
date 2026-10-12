use scraps_libs::model::{scrap::Scrap, tags::Tags};

use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::build::model::output_path::PathCollisions;
use crate::usecase::lint::rule::{LintRule, LintRuleName, LintWarning};

/// Reports what `scraps build` refuses, so it shows without building.
pub struct PathCollisionRule;

impl LintRule for PathCollisionRule {
    fn name(&self) -> LintRuleName {
        LintRuleName::PathCollision
    }

    fn check(
        &self,
        scraps: &[Scrap],
        _backlinks_map: &BacklinksMap,
        _tags: &Tags,
    ) -> Vec<LintWarning> {
        PathCollisions::find(scraps)
            .as_slice()
            .iter()
            .flat_map(|collision| {
                collision.sources.iter().map(|source| {
                    let others = collision
                        .sources
                        .iter()
                        .filter(|other| *other != source)
                        .map(String::as_str)
                        .collect::<Vec<_>>();
                    let shared_with = match collision.generated {
                        Some(page) if others.is_empty() => page.to_string(),
                        Some(page) => format!("{page} and the page of {}", others.join(", ")),
                        None => format!("the page of {}", others.join(", ")),
                    };
                    LintWarning {
                        rule_name: self.name(),
                        scrap_path: source.clone(),
                        message: format!(
                            "page path `{}` is also {shared_with}",
                            collision.output_path
                        ),
                        source: None,
                        span: None,
                    }
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(scraps: &[Scrap]) -> Vec<(String, String)> {
        let backlinks_map = BacklinksMap::new(scraps);
        let tags = Tags::new(scraps);
        PathCollisionRule
            .check(scraps, &backlinks_map, &tags)
            .into_iter()
            .map(|w| (w.scrap_path, w.message))
            .collect()
    }

    #[test]
    fn warn_each_scrap_sharing_a_page_path() {
        let scraps = vec![
            Scrap::new("a b", &None, ""),
            Scrap::new("a-b", &None, ""),
            Scrap::new("A B", &None, ""),
        ];
        assert_eq!(
            check(&scraps),
            vec![
                (
                    "A B.md".to_string(),
                    "page path `scraps/a-b.html` is also the page of a b.md, a-b.md".to_string()
                ),
                (
                    "a b.md".to_string(),
                    "page path `scraps/a-b.html` is also the page of A B.md, a-b.md".to_string()
                ),
                (
                    "a-b.md".to_string(),
                    "page path `scraps/a-b.html` is also the page of A B.md, a b.md".to_string()
                ),
            ]
        );
    }

    #[test]
    fn warn_a_scrap_on_a_generated_page_path() {
        let scraps = vec![Scrap::new("index", &None, "")];
        assert_eq!(
            check(&scraps),
            vec![(
                "index.md".to_string(),
                "page path `scraps/index.html` is also the all-scraps listing".to_string()
            )]
        );
    }

    #[test]
    fn skip_scraps_with_a_page_path_of_their_own() {
        let scraps = vec![
            Scrap::new("a", &None, ""),
            Scrap::new("a", &Some("Ctx".into()), ""),
            Scrap::new("index", &Some("Ctx".into()), ""),
        ];
        assert!(check(&scraps).is_empty());
    }
}
