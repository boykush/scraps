use std::collections::HashMap;

use scraps_libs::model::{scrap::Scrap, tags::Tags};

use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::lint::rule::{LintRule, LintRuleName, LintWarning, scrap_relative_path};

/// Flag scraps whose key cannot exist on every common filesystem: a name
/// Windows rejects, or keys that collide on a case-insensitive volume.
///
/// Opt-in: which filesystems a wiki must port to is a policy it sets.
pub struct UnportableKeyRule;

/// A check on one path segment of a key: `finds` says what is wrong with the
/// segment, `why` what that costs. To check something new, add a row.
struct SegmentCheck {
    finds: fn(&str) -> Option<String>,
    why: &'static str,
}

const SEGMENT_CHECKS: &[SegmentCheck] = &[
    SegmentCheck {
        finds: forbidden_chars,
        why: "Windows does not allow `< > : \" \\ | ? *` or control characters in a name",
    },
    SegmentCheck {
        finds: trailing_space_or_dot,
        why: "Windows drops a trailing space or dot from a name",
    },
    SegmentCheck {
        finds: reserved_device_name,
        why: "Windows reserves device names, with or without an extension",
    },
    SegmentCheck {
        finds: too_long,
        why: "most filesystems limit a name to 255 bytes",
    },
];

const WINDOWS_FORBIDDEN_CHARS: &[char] = &['<', '>', ':', '"', '\\', '|', '?', '*'];
const WINDOWS_DEVICE_NAMES: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];
const MAX_NAME_BYTES: usize = 255;

fn forbidden_chars(segment: &str) -> Option<String> {
    let mut found: Vec<char> = Vec::new();
    for c in segment.chars() {
        let forbidden = WINDOWS_FORBIDDEN_CHARS.contains(&c) || c < ' ';
        if forbidden && !found.contains(&c) {
            found.push(c);
        }
    }
    if found.is_empty() {
        return None;
    }
    let shown: Vec<String> = found
        .iter()
        .map(|c| match *c < ' ' {
            true => format!("`{}`", c.escape_default()),
            false => format!("`{c}`"),
        })
        .collect();
    Some(format!("`{segment}` contains {}", shown.join(" ")))
}

fn trailing_space_or_dot(segment: &str) -> Option<String> {
    match segment.chars().last()? {
        ' ' => Some(format!("`{segment}` ends in a space")),
        '.' => Some(format!("`{segment}` ends in a dot")),
        _ => None,
    }
}

fn reserved_device_name(segment: &str) -> Option<String> {
    let base = segment.split('.').next()?.trim_end_matches(' ');
    WINDOWS_DEVICE_NAMES
        .iter()
        .any(|name| name.eq_ignore_ascii_case(base))
        .then(|| format!("`{segment}` is a reserved device name"))
}

fn too_long(segment: &str) -> Option<String> {
    (segment.len() > MAX_NAME_BYTES).then(|| format!("a name is {} bytes long", segment.len()))
}

/// The scrap paths that share a key once case is ignored, keyed by that key.
/// Uses `str::to_lowercase`: Unicode lowercasing, not full case folding, so
/// `ß` and `SS` stay distinct.
fn paths_by_caseless_key(scraps: &[Scrap]) -> HashMap<String, Vec<String>> {
    let mut paths: HashMap<String, Vec<String>> = HashMap::new();
    for scrap in scraps {
        paths
            .entry(scrap.self_key().to_string().to_lowercase())
            .or_default()
            .push(scrap_relative_path(scrap));
    }
    paths.values_mut().for_each(|same| same.sort());
    paths
}

impl LintRule for UnportableKeyRule {
    fn name(&self) -> LintRuleName {
        LintRuleName::UnportableKey
    }

    fn check(
        &self,
        scraps: &[Scrap],
        _backlinks_map: &BacklinksMap,
        _tags: &Tags,
    ) -> Vec<LintWarning> {
        let caseless = paths_by_caseless_key(scraps);

        scraps
            .iter()
            .flat_map(|scrap| {
                let key = scrap.self_key().to_string();
                let path = scrap_relative_path(scrap);

                let mut messages: Vec<String> = Vec::new();
                for segment in key.split('/') {
                    for check in SEGMENT_CHECKS {
                        if let Some(finding) = (check.finds)(segment) {
                            messages.push(format!("{finding}: {}", check.why));
                        }
                    }
                }
                let others: Vec<String> = caseless[&key.to_lowercase()]
                    .iter()
                    .filter(|other| **other != path)
                    .map(|other| format!("`{other}`"))
                    .collect();
                if !others.is_empty() {
                    messages.push(format!(
                        "key differs only in case from {}: \
                         they cannot coexist on a case-insensitive volume",
                        others.join(", ")
                    ));
                }

                messages.into_iter().map(move |message| LintWarning {
                    rule_name: LintRuleName::UnportableKey,
                    scrap_path: path.clone(),
                    message,
                    source: None,
                    span: None,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use scraps_libs::model::context::Ctx;

    fn check(keys: &[(&str, Option<&str>)]) -> Vec<LintWarning> {
        let scraps: Vec<Scrap> = keys
            .iter()
            .map(|(title, ctx)| Scrap::new(title, &ctx.map(Ctx::from), "body"))
            .collect();
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);
        UnportableKeyRule.check(&scraps, &backlinks_map, &tags)
    }

    #[rstest]
    #[case::plain("Plain", None)]
    #[case::in_context("Blog", Some("x"))]
    #[case::hash("C#", None)]
    #[case::non_ascii("ガイド", Some("日本語"))]
    #[case::inner_dot_and_space("v1.2 notes", None)]
    #[case::device_name_as_prefix("CONSOLE", None)]
    #[case::exactly_255_bytes(&"a".repeat(255), None)]
    fn skip_portable_key(#[case] title: &str, #[case] ctx: Option<&str>) {
        assert!(check(&[(title, ctx)]).is_empty());
    }

    #[rstest]
    #[case::question_mark("What is it?", None, "`What is it?` contains `?`")]
    #[case::pipe("a|b", None, "`a|b` contains `|`")]
    #[case::several_chars("a<b>:c", None, "`a<b>:c` contains `<` `>` `:`")]
    #[case::backslash_and_quote("a\\\"b", None, "contains `\\` `\"`")]
    #[case::control_char("a\u{1}b", None, "contains `\\u{1}`")]
    #[case::char_in_context("Intro", Some("Q&A?"), "`Q&A?` contains `?`")]
    #[case::trailing_space("trailing ", None, "`trailing ` ends in a space")]
    #[case::trailing_dot("etc.", None, "`etc.` ends in a dot")]
    #[case::trailing_dot_in_context("Intro", Some("a/b."), "`b.` ends in a dot")]
    #[case::device_name("CON", None, "`CON` is a reserved device name")]
    #[case::device_name_lowercase("nul", None, "`nul` is a reserved device name")]
    #[case::device_name_with_extension("com1.notes", None, "`com1.notes` is a reserved device")]
    #[case::device_name_in_context("Intro", Some("LPT9"), "`LPT9` is a reserved device name")]
    #[case::too_long(&"a".repeat(256), None, "a name is 256 bytes long")]
    #[case::too_long_in_utf8(&"あ".repeat(86), None, "a name is 258 bytes long")]
    fn flag_segment(#[case] title: &str, #[case] ctx: Option<&str>, #[case] finding: &str) {
        let warnings = check(&[(title, ctx)]);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].rule_name, LintRuleName::UnportableKey);
        assert!(
            warnings[0].message.contains(finding),
            "{}",
            warnings[0].message
        );
    }

    #[test]
    fn report_each_finding_of_one_file() {
        let warnings = check(&[("why? ", None)]);
        assert_eq!(warnings.len(), 2);
        assert!(warnings.iter().all(|w| w.scrap_path == "why? .md"));
    }

    #[test]
    fn say_why_the_segment_does_not_port() {
        let warnings = check(&[("What is it?", None)]);
        assert!(warnings[0].message.contains("Windows does not allow"));
    }

    #[test]
    fn flag_keys_that_differ_only_in_case_once_each() {
        let warnings = check(&[
            ("Notes", None),
            ("notes", None),
            ("NOTES", None),
            ("other", None),
            ("notes", Some("x")),
        ]);
        assert_eq!(warnings.len(), 3);
        let of = |path: &str| {
            let w = warnings.iter().find(|w| w.scrap_path == path).unwrap();
            w.message.clone()
        };
        assert!(of("Notes.md").contains("differs only in case from `NOTES.md`, `notes.md`"));
        assert!(of("notes.md").contains("differs only in case from `NOTES.md`, `Notes.md`"));
        assert!(of("NOTES.md").contains("case-insensitive"));
    }

    #[test]
    fn flag_contexts_that_differ_only_in_case() {
        let warnings = check(&[("Intro", Some("Book")), ("Intro", Some("book"))]);
        assert_eq!(warnings.len(), 2);
    }

    #[test]
    fn skip_keys_that_differ_in_more_than_case() {
        assert!(check(&[("Intro", Some("Book")), ("Intro", Some("Blog"))]).is_empty());
    }
}
