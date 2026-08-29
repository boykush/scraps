/// One-line summary shown for a scrap on index and tag pages.
///
/// A leading heading is the term's expanded name (`ADR` → `Architectural
/// Decision Records`) and indexes better than any excerpt. Without one, tag
/// lines are skipped so the summary starts at prose instead of at
/// `#[[Documentation]] #[[Software Design]]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary(String);

impl Summary {
    pub fn from_md_text(md_text: &str, max_chars: usize) -> Option<Summary> {
        let mut lines = md_text.lines().map(str::trim);

        let first = lines.find(|l| !l.is_empty())?;
        let text = match heading_text(first) {
            Some(expanded) => clean(expanded),
            None => first_paragraph(std::iter::once(first).chain(lines)),
        };

        let text = truncate(&text, max_chars);
        if text.is_empty() {
            None
        } else {
            Some(Summary(text))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Markdown paragraphs are often hard-wrapped, so a summary taken from one
/// line alone stops mid-sentence. Lines are joined until the paragraph ends.
fn first_paragraph<'a>(lines: impl Iterator<Item = &'a str>) -> String {
    let mut para = String::new();
    let mut in_fence = false;

    for line in lines {
        if line.starts_with("```") || line.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }

        let ends_paragraph = line.is_empty() || is_block_break(line) || is_meta_line(line);
        let cleaned = if ends_paragraph {
            String::new()
        } else {
            clean(strip_prose_marker(line))
        };

        if cleaned.is_empty() {
            if para.is_empty() {
                continue;
            }
            break;
        }

        if needs_space(&para, &cleaned) {
            para.push(' ');
        }
        para.push_str(&cleaned);
    }

    para
}

/// A space belongs between wrapped ASCII words but not between wrapped
/// Japanese, where the line break carries no space of its own.
fn needs_space(para: &str, next: &str) -> bool {
    match (para.chars().last(), next.chars().next()) {
        (Some(prev), Some(first)) => prev.is_ascii() && first.is_ascii(),
        _ => false,
    }
}

/// A line of nothing but tags or embeds is metadata, not prose. Tags inside a
/// sentence stay, so `#[[tag]] marks a tag.` keeps its subject.
fn is_meta_line(line: &str) -> bool {
    let mut rest = line.to_string();
    for prefix in ['#', '!'] {
        rest = drop_prefixed_links(&rest, prefix);
    }
    rest.trim().is_empty()
}

fn drop_prefixed_links(line: &str, prefix: char) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == prefix && chars.get(i + 1..i + 3) == Some(&['[', '[']) {
            if let Some(end) = find_close(&chars, i + 3) {
                i = end + 2;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn is_block_break(line: &str) -> bool {
    heading_text(line).is_some()
        || (line.len() >= 3 && line.chars().all(|c| c == '-' || c == '*' || c == '_'))
}

/// `## Foo` → `Foo`. `#[[Tag]]` is not a heading: a heading needs whitespace
/// after the hashes, which is what separates the two syntaxes.
fn heading_text(line: &str) -> Option<&str> {
    let rest = line.trim_start_matches('#');
    let level = line.len() - rest.len();
    if (1..=6).contains(&level) && rest.starts_with(char::is_whitespace) {
        let text = rest.trim();
        if text.is_empty() {
            None
        } else {
            Some(text)
        }
    } else {
        None
    }
}

/// Drop list bullets and quote markers so a body that opens with a list still
/// summarises as prose.
fn strip_prose_marker(line: &str) -> &str {
    let line = line.trim_start_matches(['>', ' ']);
    for marker in ["- ", "* ", "+ "] {
        if let Some(rest) = line.strip_prefix(marker) {
            return rest.trim_start();
        }
    }
    line
}

/// Inline code fences and bold markers are noise in a one-line gloss. Single
/// `*` and `_` are left alone so identifiers like `stale_by_git` survive.
fn clean(line: &str) -> String {
    let bare = line.replace("**", "").replace("__", "").replace('`', "");
    strip_wiki_syntax(&bare)
}

/// Renders wiki syntax as the text a reader sees: `[[a|b]]` → `b`, `[[a]]` →
/// `a`, `#[[a]]` → `#a`, `![[a]]` → `a`. Any of them can be a sentence's
/// subject, so none is dropped; whole lines of them are skipped upstream.
fn strip_wiki_syntax(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len());
    let mut i = 0;

    while i < chars.len() {
        let marker = if chars.get(i + 1..i + 3) == Some(&['[', '[']) {
            match chars[i] {
                '#' => Some('#'),
                '!' => Some('!'),
                _ => None,
            }
        } else {
            None
        };
        let opens = marker.is_some() || chars.get(i..i + 2) == Some(&['[', '[']);

        if !opens {
            out.push(chars[i]);
            i += 1;
            continue;
        }

        let start = if marker.is_some() { i + 3 } else { i + 2 };
        match find_close(&chars, start) {
            Some(end) => {
                let inner: String = chars[start..end].iter().collect();
                if marker == Some('#') {
                    out.push('#');
                    out.push_str(inner.trim());
                } else {
                    out.push_str(inner.rsplit('|').next().unwrap_or(&inner).trim());
                }
                i = end + 2;
            }
            None => {
                out.push(chars[i]);
                i += 1;
            }
        }
    }

    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn find_close(chars: &[char], from: usize) -> Option<usize> {
    (from..chars.len().saturating_sub(1)).find(|&i| chars[i] == ']' && chars[i + 1] == ']')
}

/// Counts characters rather than bytes so Japanese summaries are not cut
/// mid-codepoint.
fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let head: String = text.chars().take(max_chars).collect();
    format!("{}…", head.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    // A leading heading is the expanded name, and wins over the prose below it.
    #[case(
        "## Architectural Decision Records\n\n#[[Documentation]]\n\nアーキテクチャに関する意思決定を記録する文書",
        "Architectural Decision Records"
    )]
    #[case(
        "## Large Language Model Wiki\n\n#[[LLM]] #[[Documentation]]\n\n[[Andrej Karpathy]]が提唱したパターン",
        "Large Language Model Wiki"
    )]
    // Without a heading the tag line is skipped, and links render as their text.
    #[case(
        "#[[Documentation]]\n\n[[Write the Docs]]コミュニティが提唱するアプローチ",
        "Write the Docsコミュニティが提唱するアプローチ"
    )]
    #[case(
        "#[[Documentation]] #[[Software Design]]\n\n[[ドメイン駆動設計|DDD]]において共有する言語",
        "DDDにおいて共有する言語"
    )]
    // Bodies that open straight into prose are unchanged.
    #[case(
        "個人やチームの認知容量に対する負荷のこと",
        "個人やチームの認知容量に対する負荷のこと"
    )]
    // Bullets and embeds do not leak into the summary.
    #[case("- [[Git]]を使用したバージョン管理", "Gitを使用したバージョン管理")]
    #[case("![[architecture]]\n\n実際の本文", "実際の本文")]
    #[case(
        "![[Page#Heading]] embeds a single section from another scrap.",
        "Page#Heading embeds a single section from another scrap."
    )]
    fn it_summarizes(#[case] md: &str, #[case] expected: &str) {
        let summary = Summary::from_md_text(md, 120).unwrap();
        assert_eq!(summary.as_str(), expected);
    }

    #[test]
    fn it_returns_none_when_there_is_no_prose() {
        assert_eq!(Summary::from_md_text("", 120), None);
        assert_eq!(
            Summary::from_md_text("#[[OnlyTag]]\n\n![[embed]]", 120),
            None
        );
    }

    #[test]
    fn it_truncates_on_character_boundaries() {
        let summary = Summary::from_md_text("あいうえおかきくけこ", 5).unwrap();
        assert_eq!(summary.as_str(), "あいうえお…");
    }

    #[test]
    fn it_joins_a_hard_wrapped_paragraph() {
        let md = "#[[Notation/Wiki-link]]\n\nWiki-link notation gives Markdown a typed surface: each `[[…]]` is a typed\nreference that the compiler can resolve, lint, and emit.\n\n## [[Normal Link|Normal link]]";
        let summary = Summary::from_md_text(md, 200).unwrap();
        assert_eq!(
            summary.as_str(),
            "Wiki-link notation gives Markdown a typed surface: each … is a typed reference that the compiler can resolve, lint, and emit."
        );
    }

    #[test]
    fn it_joins_wrapped_japanese_without_inserting_spaces() {
        let md = "ドメインエキスパートと開発者が\n共有する言語のこと";
        let summary = Summary::from_md_text(md, 200).unwrap();
        assert_eq!(
            summary.as_str(),
            "ドメインエキスパートと開発者が共有する言語のこと"
        );
    }

    #[test]
    fn it_stops_at_the_end_of_the_first_paragraph() {
        let md = "最初の段落。\n\n二つ目の段落は含めない。";
        let summary = Summary::from_md_text(md, 200).unwrap();
        assert_eq!(summary.as_str(), "最初の段落。");
    }

    #[test]
    fn it_skips_a_leading_code_fence() {
        let md = "```mermaid\ngraph LR\n  Source --> IR\n```\n\nビルドの流れ";
        let summary = Summary::from_md_text(md, 200).unwrap();
        assert_eq!(summary.as_str(), "ビルドの流れ");
    }

    // A tag inside a sentence is often its subject, so it renders rather than
    // being dropped; only a line made of nothing but tags is skipped.
    #[test]
    fn it_renders_an_inline_tag_as_text() {
        let md = "`#[[tag]]` marks a tag. Tags and scraps live in separate namespaces.";
        let summary = Summary::from_md_text(md, 200).unwrap();
        assert_eq!(
            summary.as_str(),
            "#tag marks a tag. Tags and scraps live in separate namespaces."
        );
    }

    #[test]
    fn it_skips_a_line_of_only_tags_and_embeds() {
        let md = "#[[Documentation]] #[[Software Design]]\n![[diagram]]\n\n本文はここから";
        let summary = Summary::from_md_text(md, 200).unwrap();
        assert_eq!(summary.as_str(), "本文はここから");
    }
}
