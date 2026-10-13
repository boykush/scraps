use scraps_libs::markdown::query::{WikiRef, wiki_refs};
use scraps_libs::model::{key::ScrapKey, scrap::Scrap, tags::Tags};

use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::lint::rule::{LintRule, LintRuleName, LintWarning, scrap_relative_path};

/// Flag scraps whose key cannot be written inside `[[ ]]` so that the link
/// lands on them. The file exists and builds, but no wiki-link reaches it.
pub struct UnlinkableKeyRule;

impl LintRule for UnlinkableKeyRule {
    fn name(&self) -> LintRuleName {
        LintRuleName::UnlinkableKey
    }

    fn check(
        &self,
        scraps: &[Scrap],
        _backlinks_map: &BacklinksMap,
        _tags: &Tags,
    ) -> Vec<LintWarning> {
        scraps
            .iter()
            .filter_map(|scrap| {
                let reason = obstacle(&scrap.self_key())?;
                Some(LintWarning {
                    rule_name: self.name(),
                    scrap_path: scrap_relative_path(scrap),
                    message: format!(
                        "key cannot be written as a wiki-link: {reason}; \
                         link to it with a Markdown link to the file, or rename the file \
                         and keep its name in a frontmatter `title`"
                    ),
                    source: None,
                    span: None,
                })
            })
            .collect()
    }
}

/// Why `[[key]]` does not land on `key`, or `None` when it does. The key is
/// read back by the compiler's own wiki-link parser instead of being checked
/// against a list of characters, so the answer follows the link syntax.
fn obstacle(key: &ScrapKey) -> Option<String> {
    let written = format!("[[{key}]]");
    let refs = wiki_refs(&written);
    let [WikiRef::Link(link)] = refs.as_slice() else {
        return Some(format!("`{written}` is not read as one wiki-link"));
    };
    let target = ScrapKey::from(link);
    if target == *key && link.heading.is_none() && link.alias.is_none() {
        return None;
    }
    // The characters only word the reason; the verdict is the round trip's.
    Some(if link.heading.is_some() && written.contains('#') {
        format!("`#` starts a heading reference, so `{written}` links to `{target}`")
    } else if link.alias.is_some() && written.contains('|') {
        format!("`|` starts an alias, so `{written}` links to `{target}`")
    } else {
        format!("`{written}` links to `{target}`")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use scraps_libs::model::context::Ctx;

    fn warnings(title: &str, ctx: Option<&str>) -> Vec<LintWarning> {
        let scraps = vec![Scrap::new(title, &ctx.map(Ctx::from), "body")];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);
        UnlinkableKeyRule.check(&scraps, &backlinks_map, &tags)
    }

    #[rstest]
    #[case::plain("Plain", None)]
    #[case::in_context("Blog", Some("x"))]
    #[case::deep_context("c", Some("a/b"))]
    #[case::question_mark("What is it?", None)]
    #[case::spaces_inside("Getting Started", None)]
    #[case::non_ascii("ガイド", Some("日本語"))]
    #[case::punctuation("C++ (2nd ed.) & more!", None)]
    fn skip_key_that_round_trips(#[case] title: &str, #[case] ctx: Option<&str>) {
        assert!(warnings(title, ctx).is_empty());
    }

    #[rstest]
    #[case::hash_in_stem("C#", None, "C#.md", "`#` starts a heading reference")]
    #[case::pipe_in_stem("a|b", None, "a|b.md", "`|` starts an alias")]
    #[case::hash_in_context("Intro", Some("C#"), "C#/Intro.md", "`#` starts a heading reference")]
    #[case::pipe_in_context("Intro", Some("a|b"), "a|b/Intro.md", "`|` starts an alias")]
    #[case::hash_in_inner_context("c", Some("a/b#"), "a/b#/c.md", "`#` starts a heading")]
    #[case::closing_brackets("a]]b", None, "a]]b.md", "`[[a]]b]]`")]
    fn flag_key_that_does_not_round_trip(
        #[case] title: &str,
        #[case] ctx: Option<&str>,
        #[case] path: &str,
        #[case] reason: &str,
    ) {
        let warnings = warnings(title, ctx);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].rule_name, LintRuleName::UnlinkableKey);
        assert_eq!(warnings[0].scrap_path, path);
        assert!(
            warnings[0].message.contains(reason),
            "{}",
            warnings[0].message
        );
        assert!(warnings[0].message.contains("Markdown link to the file"));
        assert!(warnings[0].message.contains("rename the file"));
    }

    /// Whatever the parser does with surrounding whitespace, the rule agrees
    /// with it: a name is flagged exactly when the link reads back differently.
    #[rstest]
    #[case::trailing_space("trailing ")]
    #[case::leading_space(" leading")]
    fn follow_the_parser_on_surrounding_whitespace(#[case] title: &str) {
        let key = Scrap::new(title, &None, "").self_key();
        let reads_back = matches!(
            wiki_refs(&format!("[[{key}]]")).as_slice(),
            [WikiRef::Link(link)] if ScrapKey::from(link) == key
        );
        assert_eq!(warnings(title, None).is_empty(), reads_back);
    }
}
