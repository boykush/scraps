use std::collections::{HashMap, HashSet};

use aho_corasick::{AhoCorasick, MatchKind};
use scraps_libs::markdown::query::prose;
use scraps_libs::model::{key::ScrapKey, scrap::Scrap, tags::Tags};

use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::lint::rule::{
    LintRule, LintRuleName, LintWarning, is_about_page, scrap_relative_path,
};

/// Shorter titles are mostly ordinary words and abbreviations.
const MIN_TITLE_CHARS: usize = 3;

/// Report prose that writes another scrap's title without linking to it.
pub struct UnlinkedMentionRule;

impl LintRule for UnlinkedMentionRule {
    fn name(&self) -> LintRuleName {
        LintRuleName::UnlinkedMention
    }

    fn check(
        &self,
        scraps: &[Scrap],
        _backlinks_map: &BacklinksMap,
        _tags: &Tags,
    ) -> Vec<LintWarning> {
        let titles = titles(scraps);
        let Ok(matcher) = AhoCorasick::builder()
            .match_kind(MatchKind::LeftmostLongest)
            .build(titles.iter().map(|(title, _)| title))
        else {
            return Vec::new();
        };

        scraps
            .iter()
            .flat_map(|scrap| {
                let self_key = scrap.self_key();
                let linked: HashSet<&ScrapKey> = scrap.links().iter().collect();
                let text = prose(scrap.md_text());
                let mut reported = HashSet::new();
                matcher
                    .find_iter(&text)
                    .filter_map(|found| {
                        let (title, named) = &titles[found.pattern().as_usize()];
                        let key = meant(scrap, named)?;
                        let skip = *key == self_key
                            || linked.contains(key)
                            || !whole_word(&text, found.start(), found.end())
                            || !reported.insert(key.clone());
                        (!skip).then(|| LintWarning {
                            rule_name: LintRuleName::UnlinkedMention,
                            scrap_path: scrap_relative_path(scrap),
                            message: format!("mentions \"{title}\" without linking [[{key}]]"),
                            source: Some(scrap.md_text().to_string()),
                            span: Some((found.start(), found.end())),
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}

/// A scrap a title can name, and where writing that title names it.
struct Named {
    key: ScrapKey,
    /// The context it is named in, or `None` for the whole wiki.
    within: Option<String>,
}

/// Each title long enough to match, with the scraps it can name. A title the
/// author wrote, or the file name of a scrap outside any context, names the
/// scrap anywhere. The file name of a scrap in a context names it only in
/// that context: `Kubernetes/ユーザ` is `ユーザ` to Kubernetes pages alone.
fn titles(scraps: &[Scrap]) -> Vec<(String, Vec<Named>)> {
    let mut by_title: HashMap<&str, Vec<Named>> = HashMap::new();
    for scrap in scraps.iter().filter(|scrap| !is_about_page(scrap)) {
        let within = match (scrap.frontmatter_title(), scrap.ctx()) {
            (None, Some(ctx)) => Some(ctx.to_string()),
            _ => None,
        };
        by_title.entry(scrap.title()).or_default().push(Named {
            key: scrap.self_key(),
            within,
        });
    }
    let mut titles: Vec<(String, Vec<Named>)> = by_title
        .into_iter()
        .filter(|(title, _)| title.chars().count() >= MIN_TITLE_CHARS)
        .map(|(title, named)| (title.to_string(), named))
        .collect();
    titles.sort_by(|a, b| a.0.cmp(&b.0));
    titles
}

/// The one scrap a title written in `scrap` names there. A title that could
/// name two scraps names neither.
fn meant<'a>(scrap: &Scrap, named: &'a [Named]) -> Option<&'a ScrapKey> {
    let here = scrap.ctx().as_ref().map(|ctx| ctx.to_string());
    let page = scrap.self_key().to_string();
    let mut candidates = named.iter().filter(|n| match &n.within {
        None => true,
        Some(ctx) => here.as_ref() == Some(ctx) || page == *ctx,
    });
    let key = &candidates.next()?.key;
    candidates.next().is_none().then_some(key)
}

/// A title that starts or ends in a Latin letter or digit must not run into
/// one, so `Git` stays out of `GitHub`. Scripts written without spaces
/// between words have no such edge to check.
fn whole_word(text: &str, start: usize, end: usize) -> bool {
    let word = |c: char| c.is_ascii_alphanumeric();
    let matched = &text[start..end];
    let open = !matched.starts_with(word) || !text[..start].ends_with(word);
    let close = !matched.ends_with(word) || !text[end..].starts_with(word);
    open && close
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(scraps: &[Scrap]) -> Vec<LintWarning> {
        let backlinks_map = BacklinksMap::new(scraps);
        let tags = Tags::new(scraps);
        UnlinkedMentionRule.check(scraps, &backlinks_map, &tags)
    }

    fn reported(warnings: &[LintWarning]) -> Vec<(String, String)> {
        warnings
            .iter()
            .map(|w| (w.scrap_path.clone(), w.message.clone()))
            .collect()
    }

    fn spanned(warning: &LintWarning) -> &str {
        let (start, end) = warning.span.expect("a mention should carry its span");
        &warning.source.as_deref().unwrap()[start..end]
    }

    // livt:automates livt://mapping/link-unlinked-mentions/rule/R-01/example/EX-01
    #[test]
    fn report_a_title_written_without_a_link() {
        let scraps = vec![
            Scrap::new("Rust", &None, "A language."),
            Scrap::new("notes", &None, "I am learning Rust these days."),
        ];

        let warnings = check(&scraps);

        assert_eq!(
            reported(&warnings),
            vec![(
                "notes.md".to_string(),
                "mentions \"Rust\" without linking [[Rust]]".to_string()
            )]
        );
        assert_eq!(spanned(&warnings[0]), "Rust");
    }

    // livt:automates livt://mapping/link-unlinked-mentions/rule/R-01/example/EX-02
    #[test]
    fn name_the_scrap_by_its_key_and_match_its_title() {
        let scraps = vec![
            Scrap::new(
                "orders",
                &Some("tables".into()),
                "---\ntitle: Customer Orders\n---\nOne row per order.",
            ),
            Scrap::new("report", &None, "Totals come from Customer Orders."),
        ];

        let warnings = check(&scraps);

        assert_eq!(
            reported(&warnings),
            vec![(
                "report.md".to_string(),
                "mentions \"Customer Orders\" without linking [[tables/orders]]".to_string()
            )]
        );
        assert_eq!(spanned(&warnings[0]), "Customer Orders");
    }

    // livt:automates livt://mapping/link-unlinked-mentions/rule/R-01/example/EX-03
    #[test]
    fn report_each_target_once_at_its_first_mention() {
        let scraps = vec![
            Scrap::new("Rust", &None, "A language."),
            Scrap::new("notes", &None, "Rust here.\n\nRust there."),
        ];

        let warnings = check(&scraps);

        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].span, Some((0, 4)));
    }

    // livt:automates livt://mapping/link-unlinked-mentions/rule/R-02/example/EX-01
    #[test]
    fn skip_a_scrap_that_already_links_the_target() {
        let scraps = vec![
            Scrap::new("Rust", &None, "A language."),
            Scrap::new("wiki", &None, "See [[Rust]]. Later, Rust again."),
            Scrap::new("md", &None, "See [it](Rust.md). Later, Rust again."),
        ];

        assert!(check(&scraps).is_empty());
    }

    // livt:automates livt://mapping/link-unlinked-mentions/rule/R-02/example/EX-02
    #[test]
    fn skip_a_title_inside_code_links_urls_or_frontmatter() {
        let scraps = vec![
            Scrap::new("Rust", &None, "A language."),
            Scrap::new("Gopher", &None, "Another."),
            Scrap::new(
                "notes",
                &None,
                "---\ndescription: about Rust\n---\nRun `Rust` or [Gopher](Gopher.md) via [[Gopher|Rust]].\n\n```\nRust\n```\n\nhttps://example.com/Rust\n",
            ),
        ];

        assert!(check(&scraps).is_empty());
    }

    // livt:automates livt://mapping/link-unlinked-mentions/rule/R-02/example/EX-03
    #[test]
    fn skip_a_scrap_mentioning_its_own_title() {
        let scraps = vec![Scrap::new("Rust", &None, "Rust is a language.")];

        assert!(check(&scraps).is_empty());
    }

    // livt:automates livt://mapping/link-unlinked-mentions/rule/R-03/example/EX-01
    #[test]
    fn match_a_latin_title_as_a_whole_word_and_case() {
        let scraps = vec![
            Scrap::new("Git", &None, "A VCS."),
            Scrap::new(
                "notes",
                &None,
                "GitHub hosts it; git is lowercase; Gitter too.",
            ),
        ];

        assert!(check(&scraps).is_empty());
    }

    // livt:automates livt://mapping/link-unlinked-mentions/rule/R-03/example/EX-02
    #[test]
    fn match_a_title_without_word_breaks_anywhere_in_the_text() {
        let scraps = vec![
            Scrap::new(
                "認可",
                &Some("auth".into()),
                "---\ntitle: 認可制御\n---\n本文。",
            ),
            Scrap::new("notes", &None, "ここで認可制御を導入した。"),
        ];

        let warnings = check(&scraps);

        assert_eq!(warnings.len(), 1);
        assert_eq!(spanned(&warnings[0]), "認可制御");
    }

    // livt:automates livt://mapping/link-unlinked-mentions/rule/R-03/example/EX-03
    #[test]
    fn prefer_the_longest_title_at_a_place() {
        let scraps = vec![
            Scrap::new("Claude", &None, "A model."),
            Scrap::new("Claude Code", &None, "A CLI."),
            Scrap::new("notes", &None, "I use Claude Code daily."),
        ];

        let warnings = check(&scraps);

        assert_eq!(
            reported(&warnings),
            vec![(
                "notes.md".to_string(),
                "mentions \"Claude Code\" without linking [[Claude Code]]".to_string()
            )]
        );
    }

    // livt:automates livt://mapping/link-unlinked-mentions/rule/R-04/example/EX-01
    #[test]
    fn skip_titles_too_short_to_tell_from_ordinary_words() {
        let scraps = vec![
            Scrap::new("Go", &None, "A language."),
            Scrap::new("notes", &None, "Go for it."),
        ];

        assert!(check(&scraps).is_empty());
    }

    // livt:automates livt://mapping/link-unlinked-mentions/rule/R-04/example/EX-02
    #[test]
    fn skip_a_title_two_scraps_share_where_both_could_be_meant() {
        let scraps = vec![
            Scrap::new("a", &None, "---\ntitle: Orders\n---\nOne."),
            Scrap::new("b", &None, "---\ntitle: Orders\n---\nTwo."),
            Scrap::new("notes", &None, "Some Orders here."),
        ];

        assert!(check(&scraps).is_empty());
    }

    // livt:automates livt://mapping/link-unlinked-mentions/rule/R-06/example/EX-01
    #[test]
    fn match_a_file_named_title_in_a_context_only_inside_that_context() {
        let scraps = vec![
            Scrap::new("ユーザ", &Some("Kubernetes".into()), "認証される主体。"),
            Scrap::new("notes", &None, "ユーザが増えた。"),
            Scrap::new("Pod", &Some("Kubernetes".into()), "ユーザが作る。"),
            Scrap::new("Kubernetes", &None, "ユーザを持つ。"),
        ];

        let mut warnings = reported(&check(&scraps));
        warnings.sort();

        assert_eq!(
            warnings,
            vec![
                (
                    "Kubernetes.md".to_string(),
                    "mentions \"ユーザ\" without linking [[Kubernetes/ユーザ]]".to_string()
                ),
                (
                    "Kubernetes/Pod.md".to_string(),
                    "mentions \"ユーザ\" without linking [[Kubernetes/ユーザ]]".to_string()
                ),
            ]
        );
    }

    // livt:automates livt://mapping/link-unlinked-mentions/rule/R-06/example/EX-02
    #[test]
    fn tell_scraps_that_share_a_file_name_apart_by_their_context() {
        let scraps = vec![
            Scrap::new("Service", &Some("DDD".into()), "Domain."),
            Scrap::new("Service", &Some("Kubernetes".into()), "Network."),
            Scrap::new("notes", &None, "A Service here."),
            Scrap::new("Pod", &Some("Kubernetes".into()), "Exposed by a Service."),
        ];

        assert_eq!(
            reported(&check(&scraps)),
            vec![(
                "Kubernetes/Pod.md".to_string(),
                "mentions \"Service\" without linking [[Kubernetes/Service]]".to_string()
            )]
        );
    }

    // livt:automates livt://mapping/link-unlinked-mentions/rule/R-02/example/EX-04
    #[test]
    fn skip_the_readme_the_site_renders_as_its_about_page() {
        let scraps = vec![
            Scrap::new("README", &None, "About this wiki."),
            Scrap::new("notes", &None, "See the README for setup."),
        ];

        assert!(check(&scraps).is_empty());
    }
}
