use std::collections::HashMap;

use scraps_libs::model::{key::ScrapKey, scrap::Scrap, tags::Tags};

use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::lint::rule::{LintRule, LintRuleName, LintWarning, scrap_relative_path};

/// Detect a `sources` entry naming, by its URL, something another scrap is
/// about: that scrap's top-level `resource` is the same string, so the entry
/// can point at the scrap by path instead.
pub struct SourceHasPageRule;

impl LintRule for SourceHasPageRule {
    fn name(&self) -> LintRuleName {
        LintRuleName::SourceHasPage
    }

    fn check(
        &self,
        scraps: &[Scrap],
        _backlinks_map: &BacklinksMap,
        _tags: &Tags,
    ) -> Vec<LintWarning> {
        let mut pages: HashMap<&str, ScrapKey> = HashMap::new();
        for scrap in scraps {
            if let Some(resource) = scrap.resource() {
                pages.entry(resource).or_insert_with(|| scrap.self_key());
            }
        }

        scraps
            .iter()
            .flat_map(|scrap| {
                let self_key = scrap.self_key();
                scrap
                    .sources()
                    .into_iter()
                    .filter_map(|source| {
                        let page = pages.get(source.resource.as_str())?;
                        (*page != self_key).then(|| (source.resource, page.clone()))
                    })
                    .map(|(resource, page)| LintWarning {
                        rule_name: self.name(),
                        scrap_path: scrap_relative_path(scrap),
                        message: format!("source {resource} has a page: write /{page}.md"),
                        source: Some(scrap.md_text().to_string()),
                        span: scrap
                            .md_text()
                            .find(&resource)
                            .map(|start| (start, start + resource.len())),
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(scraps: &[Scrap]) -> Vec<String> {
        SourceHasPageRule
            .check(scraps, &BacklinksMap::new(scraps), &Tags::new(scraps))
            .into_iter()
            .map(|w| w.message)
            .collect()
    }

    #[test]
    fn detect_a_source_url_another_scrap_is_about() {
        let scraps = vec![
            Scrap::new(
                "Intro",
                &Some("Blog".into()),
                "---\nresource: https://example.com/intro\n---\n",
            ),
            Scrap::new(
                "OKF",
                &None,
                "---\nsources:\n  - resource: https://example.com/intro\n  - resource: https://example.com/spec\n---\n",
            ),
        ];

        assert_eq!(
            check(&scraps),
            vec!["source https://example.com/intro has a page: write /Blog/Intro.md"]
        );
    }

    #[test]
    fn ignore_the_scrap_naming_its_own_resource() {
        let scraps = vec![Scrap::new(
            "Intro",
            &None,
            "---\nresource: https://example.com/intro\nsources:\n  - resource: https://example.com/intro\n---\n",
        )];

        assert!(check(&scraps).is_empty());
    }

    #[test]
    fn ignore_a_source_already_written_as_a_path() {
        let scraps = vec![
            Scrap::new(
                "Intro",
                &None,
                "---\nresource: https://example.com/intro\n---\n",
            ),
            Scrap::new(
                "OKF",
                &None,
                "---\nsources:\n  - resource: /Intro.md\n---\n",
            ),
        ];

        assert!(check(&scraps).is_empty());
    }
}
