use comrak::{
    nodes::{AstNode, NodeValue, NodeWikiLink},
    options::Extension,
    parse_document, Arena, Options,
};

use crate::model::key::ScrapKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiLinkRef {
    pub ctx_path: Vec<String>,
    pub title: String,
    pub heading: Option<String>,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbedRef {
    pub ctx_path: Vec<String>,
    pub title: String,
    pub heading: Option<String>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagOccurrence {
    pub path: Vec<String>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Open,
    Done,
    Deferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskItem {
    pub status: TaskStatus,
    pub text: String,
    pub line: usize,
}

fn options() -> Options<'static> {
    let mut opts = Options::default();
    opts.extension.wikilinks_title_after_pipe = true;
    opts.extension.tasklist = true;
    opts.parse.relaxed_tasklist_matching = true;
    opts
}

fn collect_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut s = String::new();
    for d in node.descendants() {
        if let NodeValue::Text(t) = &d.data().value {
            s.push_str(t);
        }
    }
    s
}

fn parse_wikilink_url(url: &str) -> (Vec<String>, String, Option<String>) {
    let (path, heading) = match url.split_once('#') {
        Some((p, h)) => (p.to_string(), Some(h.to_string())),
        None => (url.to_string(), None),
    };
    let mut parts: Vec<String> = path.split('/').map(|s| s.to_string()).collect();
    let title = parts.pop().unwrap_or_default();
    (parts, title, heading)
}

pub fn wikilinks(text: &str) -> Vec<WikiLinkRef> {
    let arena = Arena::new();
    let opts = options();
    let root = parse_document(&arena, text, &opts);
    root.descendants()
        .filter_map(|node| match &node.data().value {
            NodeValue::WikiLink(NodeWikiLink { url }) if !url.is_empty() => {
                let (ctx_path, title, heading) = parse_wikilink_url(url);
                let label = collect_text(node);
                let display = match &heading {
                    Some(h) => format!("{}#{}", url_path(&ctx_path, &title), h),
                    None => url_path(&ctx_path, &title),
                };
                let alias = if label == display { None } else { Some(label) };
                Some(WikiLinkRef {
                    ctx_path,
                    title,
                    heading,
                    alias,
                })
            }
            _ => None,
        })
        .collect()
}

fn url_path(ctx_path: &[String], title: &str) -> String {
    if ctx_path.is_empty() {
        title.to_string()
    } else {
        format!("{}/{}", ctx_path.join("/"), title)
    }
}

impl From<&WikiLinkRef> for ScrapKey {
    fn from(w: &WikiLinkRef) -> Self {
        ScrapKey::from_path_str(&url_path(&w.ctx_path, &w.title))
    }
}

fn line_starts(text: &str) -> Vec<usize> {
    let mut v = vec![0];
    for (i, b) in text.bytes().enumerate() {
        if b == b'\n' {
            v.push(i + 1);
        }
    }
    v
}

fn byte_to_line(starts: &[usize], byte: usize) -> usize {
    starts.partition_point(|&s| s <= byte)
}

fn line_col_to_byte(starts: &[usize], line: usize, col: usize) -> usize {
    let li = line.saturating_sub(1);
    let base = starts.get(li).copied().unwrap_or(0);
    base + col.saturating_sub(1)
}

fn code_byte_ranges<'a>(root: &'a AstNode<'a>, starts: &[usize]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    for n in root.descendants() {
        let in_code = matches!(
            &n.data().value,
            NodeValue::CodeBlock(_) | NodeValue::Code(_)
        );
        if !in_code {
            continue;
        }
        let pos = n.data().sourcepos;
        let s = line_col_to_byte(starts, pos.start.line, pos.start.column);
        let e = line_col_to_byte(starts, pos.end.line, pos.end.column) + 1;
        ranges.push((s, e));
    }
    ranges.sort_by_key(|r| r.0);
    ranges
}

fn in_code(ranges: &[(usize, usize)], byte: usize) -> bool {
    ranges.iter().any(|(s, e)| *s <= byte && byte < *e)
}

pub fn embeds(text: &str) -> Vec<EmbedRef> {
    let arena = Arena::new();
    let opts = options();
    let root = parse_document(&arena, text, &opts);
    let starts = line_starts(text);
    let codes = code_byte_ranges(root, &starts);
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 4 < bytes.len() {
        if bytes[i] == b'!' && bytes[i + 1] == b'[' && bytes[i + 2] == b'[' && !in_code(&codes, i) {
            let inner_start = i + 3;
            let mut j = inner_start;
            let mut found = false;
            while j + 1 < bytes.len() {
                if bytes[j] == b'\n' {
                    break;
                }
                if bytes[j] == b']' && bytes[j + 1] == b']' {
                    found = true;
                    break;
                }
                j += 1;
            }
            if found {
                let inner = &text[inner_start..j];
                if !inner.is_empty() && !inner.contains('|') {
                    let (ctx_path, title, heading) = parse_wikilink_url(inner);
                    if !title.is_empty() {
                        let line = byte_to_line(&starts, i);
                        out.push(EmbedRef {
                            ctx_path,
                            title,
                            heading,
                            line,
                        });
                    }
                }
                i = j + 2;
                continue;
            }
        }
        i += 1;
    }
    out
}

pub fn tags(text: &str) -> Vec<TagOccurrence> {
    let arena = Arena::new();
    let opts = options();
    let root = parse_document(&arena, text, &opts);
    let starts = line_starts(text);
    let codes = code_byte_ranges(root, &starts);
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 4 < bytes.len() {
        if bytes[i] == b'#' && bytes[i + 1] == b'[' && bytes[i + 2] == b'[' && !in_code(&codes, i) {
            let inner_start = i + 3;
            let mut j = inner_start;
            let mut found = false;
            while j + 1 < bytes.len() {
                if bytes[j] == b'\n' {
                    break;
                }
                if bytes[j] == b']' && bytes[j + 1] == b']' {
                    found = true;
                    break;
                }
                j += 1;
            }
            if found {
                let inner = &text[inner_start..j];
                if !inner.is_empty() {
                    let path: Vec<String> = inner.split('/').map(|s| s.to_string()).collect();
                    if path.iter().all(|s| !s.is_empty()) {
                        let line = byte_to_line(&starts, i);
                        out.push(TagOccurrence { path, line });
                    }
                }
                i = j + 2;
                continue;
            }
        }
        i += 1;
    }
    out
}

fn task_item_body<'a>(node: &'a AstNode<'a>) -> String {
    let mut s = String::new();
    for child in node.children() {
        if !matches!(child.data().value, NodeValue::Paragraph) {
            continue;
        }
        for d in child.descendants() {
            if let NodeValue::Text(t) = &d.data().value {
                s.push_str(t);
            }
        }
    }
    s
}

pub fn task_items(text: &str) -> Vec<TaskItem> {
    let arena = Arena::new();
    let opts = options();
    let root = parse_document(&arena, text, &opts);
    let mut out = Vec::new();
    for node in root.descendants() {
        let NodeValue::TaskItem(item) = &node.data().value else {
            continue;
        };
        let status = match item.symbol {
            None | Some(' ') => TaskStatus::Open,
            Some('x') | Some('X') => TaskStatus::Done,
            Some('-') => TaskStatus::Deferred,
            _ => continue,
        };
        let body = task_item_body(node);
        let line = node.data().sourcepos.start.line;
        out.push(TaskItem {
            status,
            text: body,
            line,
        });
    }
    out
}

fn gfm_slug(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
        } else if c.is_whitespace() || c == '-' {
            out.push('-');
        }
    }
    let mut collapsed = String::with_capacity(out.len());
    let mut last_dash = false;
    for c in out.chars() {
        if c == '-' {
            if last_dash {
                continue;
            }
            last_dash = true;
        } else {
            last_dash = false;
        }
        collapsed.push(c);
    }
    collapsed.trim_matches('-').to_string()
}

fn line_byte_offset(starts: &[usize], total_len: usize, line: usize) -> usize {
    if line == 0 {
        return 0;
    }
    starts.get(line - 1).copied().unwrap_or(total_len)
}

struct HeadingInfo {
    start_line: usize,
    end_line: usize,
    level: u8,
    slug: String,
}

pub fn section<'a>(text: &'a str, heading_slug: &str) -> Option<&'a str> {
    if text.is_empty() {
        return None;
    }
    let arena = Arena::new();
    let opts = options();
    let root = parse_document(&arena, text, &opts);
    let starts = line_starts(text);
    let total = text.len();

    let mut hs: Vec<HeadingInfo> = Vec::new();
    for n in root.children() {
        if let NodeValue::Heading(h) = &n.data().value {
            let pos = n.data().sourcepos;
            let label = collect_text(n);
            hs.push(HeadingInfo {
                start_line: pos.start.line,
                end_line: pos.end.line,
                level: h.level as u8,
                slug: gfm_slug(&label),
            });
        }
    }

    let idx = hs.iter().position(|h| h.slug == heading_slug)?;
    let target_level = hs[idx].level;
    let body_start_line = hs[idx].end_line + 1;
    let body_end_line = hs[idx + 1..]
        .iter()
        .find(|h| h.level <= target_level)
        .map(|h| h.start_line);

    let start_byte = line_byte_offset(&starts, total, body_start_line);
    let end_byte = match body_end_line {
        Some(line) => line_byte_offset(&starts, total, line),
        None => total,
    };
    if start_byte > end_byte || start_byte > total {
        return Some("");
    }
    Some(&text[start_byte..end_byte])
}

#[cfg(test)]
mod wikilinks_tests {
    use super::*;
    use rstest::rstest;

    fn link(
        ctx_path: &[&str],
        title: &str,
        heading: Option<&str>,
        alias: Option<&str>,
    ) -> WikiLinkRef {
        WikiLinkRef {
            ctx_path: ctx_path.iter().map(|s| s.to_string()).collect(),
            title: title.to_string(),
            heading: heading.map(|s| s.to_string()),
            alias: alias.map(|s| s.to_string()),
        }
    }

    #[rstest]
    #[case::basic("[[a]]", vec![link(&[], "a", None, None)])]
    #[case::ctx_title("[[Book/Test-driven development]]",
        vec![link(&["Book"], "Test-driven development", None, None)])]
    #[case::deep_ctx("[[a/b/c]]", vec![link(&["a", "b"], "c", None, None)])]
    #[case::alias_pipe("[[Domain Driven Design|DDD]]",
        vec![link(&[], "Domain Driven Design", None, Some("DDD"))])]
    #[case::alias_special_chars("[[topic|hello, world!]]",
        vec![link(&[], "topic", None, Some("hello, world!"))])]
    #[case::heading_only("[[topic#section]]",
        vec![link(&[], "topic", Some("section"), None)])]
    #[case::ctx_heading_alias("[[Person/Eric Evans#bio|Eric]]",
        vec![link(&["Person"], "Eric Evans", Some("bio"), Some("Eric"))])]
    #[case::contain_space("[[contain space]]", vec![link(&[], "contain space", None, None)])]
    #[case::ctx_alias("[[Person/Eric Evans|Eric Evans]]",
        vec![link(&["Person"], "Eric Evans", None, Some("Eric Evans"))])]
    fn it_wikilinks_base(#[case] input: &str, #[case] expected: Vec<WikiLinkRef>) {
        assert_eq!(wikilinks(input), expected);
    }

    #[rstest]
    #[case::japanese("[[日本語タイトル]]",
        vec![link(&[], "日本語タイトル", None, None)])]
    #[case::emoji("[[🚀rocket]]", vec![link(&[], "🚀rocket", None, None)])]
    #[case::cjk_mix("[[Book/Domain駆動]]",
        vec![link(&["Book"], "Domain駆動", None, None)])]
    #[case::japanese_alias("[[Domain Driven Design|ドメイン駆動設計]]",
        vec![link(&[], "Domain Driven Design", None, Some("ドメイン駆動設計"))])]
    fn it_wikilinks_unicode(#[case] input: &str, #[case] expected: Vec<WikiLinkRef>) {
        assert_eq!(wikilinks(input), expected);
    }

    #[test]
    fn it_wikilinks_lf_joined() {
        let input = "[[a]]\n[[b]]\n[[c]]";
        let res = wikilinks(input);
        assert_eq!(res.len(), 3);
    }

    #[test]
    fn it_wikilinks_crlf_joined() {
        let input = "[[a]]\r\n[[b]]\r\n[[c]]";
        let res = wikilinks(input);
        assert_eq!(res.len(), 3);
    }

    #[test]
    fn it_wikilinks_no_trailing_newline() {
        let input = "[[a]] [[b]]";
        let res = wikilinks(input);
        assert_eq!(res.len(), 2);
    }

    #[test]
    fn it_wikilinks_blank_line_gap() {
        let input = "[[a]]\n\n\n[[b]]";
        let res = wikilinks(input);
        assert_eq!(res.len(), 2);
    }

    #[rstest]
    #[case::inline_code("`[[x]]`")]
    #[case::fenced_code("```\n[[x]]\n```")]
    #[case::indented_code("    [[x]]")]
    #[case::tilde_fence("~~~\n[[x]]\n~~~")]
    fn it_wikilinks_excludes_code(#[case] input: &str) {
        assert!(wikilinks(input).is_empty());
    }

    #[rstest]
    #[case::empty_brackets("[[]]")]
    #[case::only_open("[[only open")]
    #[case::only_close("only close]]")]
    #[case::single_brackets("[single]")]
    #[case::space_between("[ [space] ]")]
    #[case::multi_pipe("[[a|b|c]]")]
    fn it_wikilinks_invalid(#[case] input: &str) {
        assert!(wikilinks(input).is_empty());
    }

    #[test]
    fn it_wikilinks_preserves_duplicates() {
        let input = "[[a]] [[a]] [[a]] [[b]] [[b]] [[c]]";
        let result = wikilinks(input);
        assert_eq!(result.len(), 6);
        let a = link(&[], "a", None, None);
        assert_eq!(result.iter().filter(|w| **w == a).count(), 3);
    }

    #[test]
    fn it_wikilinks_composite_document() {
        let input = "\
# heading

[[plain]] and [[Book/TDD|TDD book]] in body.

```
[[ignored]]
```

`[[also ignored]]` near [[Person/Eric#bio]].
";
        let res = wikilinks(input);
        assert_eq!(res.len(), 3);
        assert_eq!(res[0], link(&[], "plain", None, None));
        assert_eq!(res[1], link(&["Book"], "TDD", None, Some("TDD book")));
        assert_eq!(res[2], link(&["Person"], "Eric", Some("bio"), None));
    }

    #[rstest]
    #[case::no_ctx(link(&[], "a", None, None), "a", None)]
    #[case::single_ctx(link(&["ctx"], "a", None, None), "a", Some("ctx"))]
    #[case::two_deep(link(&["a", "b"], "c", None, None), "b/c", Some("a"))]
    fn it_wikilinkref_into_scrapkey(
        #[case] w: WikiLinkRef,
        #[case] expected_title: &str,
        #[case] expected_ctx: Option<&str>,
    ) {
        use crate::model::context::Ctx;
        use crate::model::title::Title;
        let key: ScrapKey = (&w).into();
        assert_eq!(Title::from(&key), expected_title.into());
        assert_eq!(Option::<Ctx>::from(&key), expected_ctx.map(|c| c.into()));
    }
}

#[cfg(test)]
mod embeds_tests {
    use super::*;
    use rstest::rstest;

    fn embed(ctx_path: &[&str], title: &str, heading: Option<&str>, line: usize) -> EmbedRef {
        EmbedRef {
            ctx_path: ctx_path.iter().map(|s| s.to_string()).collect(),
            title: title.to_string(),
            heading: heading.map(|s| s.to_string()),
            line,
        }
    }

    #[rstest]
    #[case::basic("![[name]]", vec![embed(&[], "name", None, 1)])]
    #[case::ctx("![[Book/name]]", vec![embed(&["Book"], "name", None, 1)])]
    #[case::heading("![[name#h]]", vec![embed(&[], "name", Some("h"), 1)])]
    #[case::ctx_heading("![[a/b#h]]", vec![embed(&["a"], "b", Some("h"), 1)])]
    #[case::multiple_in_paragraph(
        "![[a]] and ![[b]] and ![[c]]",
        vec![embed(&[], "a", None, 1), embed(&[], "b", None, 1), embed(&[], "c", None, 1)]
    )]
    #[case::japanese("![[日本語]]", vec![embed(&[], "日本語", None, 1)])]
    #[case::emoji("![[🚀name]]", vec![embed(&[], "🚀name", None, 1)])]
    fn it_embeds_base(#[case] input: &str, #[case] expected: Vec<EmbedRef>) {
        assert_eq!(embeds(input), expected);
    }

    #[rstest]
    #[case::image_syntax("![alt](https://example.com/x.png)")]
    #[case::bare_bang("Just a ! mark and [[link]]")]
    #[case::bang_then_newline("!\n[[name]]")]
    #[case::bang_then_blankline("!\n\n[[name]]")]
    #[case::wikilink_no_bang("[[name]]")]
    #[case::space_between("! [[name]]")]
    fn it_embeds_non_match(#[case] input: &str) {
        assert!(embeds(input).is_empty());
    }

    #[rstest]
    #[case::inline_code("`![[x]]`")]
    #[case::fenced_code("```\n![[x]]\n```")]
    #[case::indented_code("    ![[x]]")]
    #[case::quoted_code("> `![[x]]`")]
    fn it_embeds_excludes_code(#[case] input: &str) {
        assert!(embeds(input).is_empty());
    }

    #[test]
    fn it_embeds_line_numbers_lf() {
        let input = "first line\n\n![[a]]\n\nthird block\n\n![[b]]";
        let res = embeds(input);
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].line, 3);
        assert_eq!(res[1].line, 7);
    }

    #[test]
    fn it_embeds_line_numbers_crlf() {
        let input = "first\r\n\r\n![[a]]\r\n\r\nlast\r\n\r\n![[b]]";
        let res = embeds(input);
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].line, 3);
        assert_eq!(res[1].line, 7);
    }

    #[test]
    fn it_embeds_in_nested_list() {
        let input = "- top\n  - ![[nested]]\n";
        let res = embeds(input);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].title, "nested");
    }

    #[test]
    fn it_embeds_composite_document() {
        let input = "\
# title

intro ![[a]] inline.

```
![[ignored]]
```

`![[also-ignored]]` near ![[Person/Eric#bio]] body.
";
        let res = embeds(input);
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].title, "a");
        assert_eq!(res[1].title, "Eric");
        assert_eq!(res[1].ctx_path, vec!["Person".to_string()]);
        assert_eq!(res[1].heading.as_deref(), Some("bio"));
    }
}

#[cfg(test)]
mod tags_tests {
    use super::*;
    use rstest::rstest;

    fn tag(path: &[&str], line: usize) -> TagOccurrence {
        TagOccurrence {
            path: path.iter().map(|s| s.to_string()).collect(),
            line,
        }
    }

    #[rstest]
    #[case::depth1("#[[a]]", vec![tag(&["a"], 1)])]
    #[case::depth2("#[[a/b]]", vec![tag(&["a", "b"], 1)])]
    #[case::depth3("#[[a/b/c]]", vec![tag(&["a", "b", "c"], 1)])]
    #[case::multi_per_line(
        "#[[a]] and #[[b]] and #[[c]]",
        vec![tag(&["a"], 1), tag(&["b"], 1), tag(&["c"], 1)]
    )]
    fn it_tags_base(#[case] input: &str, #[case] expected: Vec<TagOccurrence>) {
        assert_eq!(tags(input), expected);
    }

    #[rstest]
    #[case::inline_code("`#[[t]]`")]
    #[case::fenced_code("```\n#[[t]]\n```")]
    #[case::indented_code("    #[[t]]")]
    #[case::tilde_fence("~~~\n#[[t]]\n~~~")]
    fn it_tags_excludes_code(#[case] input: &str) {
        assert!(tags(input).is_empty());
    }

    #[test]
    fn it_tags_in_heading_text() {
        assert!(tags("# heading").is_empty());
        assert!(tags("## heading").is_empty());
        assert!(tags("# [[link]]").is_empty());
    }

    #[test]
    fn it_tags_consecutive_hashes_emit_one() {
        let r1 = tags("##[[t]]");
        assert_eq!(r1, vec![tag(&["t"], 1)]);
        let r2 = tags("###[[t]]");
        assert_eq!(r2, vec![tag(&["t"], 1)]);
    }

    #[rstest]
    #[case::empty_inner("#[[]]")]
    #[case::unterminated("#[[a")]
    #[case::heading_link("# [[x]]")]
    #[case::nested_brackets("#[[]]")]
    #[case::trailing_slash("#[[a/]]")]
    #[case::leading_slash("#[[/a]]")]
    fn it_tags_invalid(#[case] input: &str) {
        assert!(tags(input).is_empty());
    }

    #[rstest]
    #[case::japanese("#[[日本語]]", vec![tag(&["日本語"], 1)])]
    #[case::japanese_ctx("#[[ctx/日本]]", vec![tag(&["ctx", "日本"], 1)])]
    fn it_tags_unicode(#[case] input: &str, #[case] expected: Vec<TagOccurrence>) {
        assert_eq!(tags(input), expected);
    }

    #[test]
    fn it_tags_line_numbers_lf() {
        let input = "first\n\n#[[a]]\n\nlast\n\n#[[b]]";
        let res = tags(input);
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].line, 3);
        assert_eq!(res[1].line, 7);
    }

    #[test]
    fn it_tags_line_numbers_crlf() {
        let input = "first\r\n\r\n#[[a]]\r\n\r\nlast\r\n\r\n#[[b]]";
        let res = tags(input);
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].line, 3);
        assert_eq!(res[1].line, 7);
    }

    #[test]
    fn it_tags_composite_document() {
        let input = "\
# title

body has #[[topic]] and ##[[multi]] and #[[a/b/c]].

```
#[[ignored]]
```

`#[[also-ignored]]` then #[[final]] last.
";
        let res = tags(input);
        let paths: Vec<Vec<String>> = res.iter().map(|t| t.path.clone()).collect();
        assert_eq!(
            paths,
            vec![
                vec!["topic".to_string()],
                vec!["multi".to_string()],
                vec!["a".to_string(), "b".to_string(), "c".to_string()],
                vec!["final".to_string()],
            ]
        );
    }
}

#[cfg(test)]
mod task_items_tests {
    use super::*;
    use rstest::rstest;

    fn item(status: TaskStatus, text: &str, line: usize) -> TaskItem {
        TaskItem {
            status,
            text: text.to_string(),
            line,
        }
    }

    #[rstest]
    #[case::open("- [ ] open", vec![item(TaskStatus::Open, "open", 1)])]
    #[case::done_lower("- [x] done", vec![item(TaskStatus::Done, "done", 1)])]
    #[case::done_upper("- [X] DONE", vec![item(TaskStatus::Done, "DONE", 1)])]
    #[case::deferred("- [-] deferred", vec![item(TaskStatus::Deferred, "deferred", 1)])]
    fn it_task_items_status(#[case] input: &str, #[case] expected: Vec<TaskItem>) {
        assert_eq!(task_items(input), expected);
    }

    #[rstest]
    #[case::slash("- [/] other")]
    #[case::question("- [?] huh")]
    #[case::dot("- [.] dot")]
    fn it_task_items_unsupported_skipped(#[case] input: &str) {
        assert!(task_items(input).is_empty());
    }

    #[test]
    fn it_task_items_nested_list() {
        let input = "- [ ] top\n  - [x] sub\n  - [-] sub-deferred\n";
        let res = task_items(input);
        assert_eq!(res.len(), 3);
        assert_eq!(res[0].status, TaskStatus::Open);
        assert_eq!(res[0].text, "top");
        assert_eq!(res[1].status, TaskStatus::Done);
        assert_eq!(res[1].text, "sub");
        assert_eq!(res[2].status, TaskStatus::Deferred);
        assert_eq!(res[2].text, "sub-deferred");
    }

    #[test]
    fn it_task_items_in_blockquote() {
        let input = "> - [ ] quoted\n> - [x] also\n";
        let res = task_items(input);
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].status, TaskStatus::Open);
        assert_eq!(res[1].status, TaskStatus::Done);
    }

    #[test]
    fn it_task_items_mixed_with_plain_bullets() {
        let input = "- plain bullet\n- [ ] task\n- another plain\n- [x] done\n";
        let res = task_items(input);
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].text, "task");
        assert_eq!(res[1].text, "done");
    }

    #[test]
    fn it_task_items_inline_formatting_text_is_plain() {
        let input = "- [ ] read **the** book";
        let res = task_items(input);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].text, "read the book");
    }

    #[test]
    fn it_task_items_with_wikilink_in_text() {
        let input = "- [ ] link to [[topic]] now";
        let res = task_items(input);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].text, "link to topic now");
    }

    #[test]
    fn it_task_items_line_numbers_lf() {
        let input = "- [ ] first\n- [x] second\n- [-] third";
        let res = task_items(input);
        assert_eq!(res.len(), 3);
        assert_eq!(res[0].line, 1);
        assert_eq!(res[1].line, 2);
        assert_eq!(res[2].line, 3);
    }

    #[test]
    fn it_task_items_line_numbers_crlf() {
        let input = "- [ ] first\r\n- [x] second\r\n- [-] third";
        let res = task_items(input);
        assert_eq!(res.len(), 3);
        assert_eq!(res[0].line, 1);
        assert_eq!(res[1].line, 2);
        assert_eq!(res[2].line, 3);
    }

    #[test]
    fn it_task_items_composite_document() {
        let input = "\
# Title

- [ ] todo
  - [x] sub done
- regular bullet
- [-] deferred

> - [ ] quoted

```
- [ ] in-code
```
";
        let res = task_items(input);
        assert_eq!(res.len(), 4);
        assert_eq!(res[0].status, TaskStatus::Open);
        assert_eq!(res[0].text, "todo");
        assert_eq!(res[1].status, TaskStatus::Done);
        assert_eq!(res[2].status, TaskStatus::Deferred);
        assert_eq!(res[3].status, TaskStatus::Open);
        assert_eq!(res[3].text, "quoted");
    }
}

#[cfg(test)]
mod section_tests {
    use super::*;

    #[test]
    fn it_section_basic_h2() {
        let input = "## Hello\n\nbody line\n";
        assert_eq!(section(input, "hello"), Some("\nbody line\n"));
    }

    #[test]
    fn it_section_punctuation_in_heading() {
        let input = "## Hello, World!\n\nbody\n";
        assert_eq!(section(input, "hello-world"), Some("\nbody\n"));
    }

    #[test]
    fn it_section_japanese_heading() {
        let input = "## 見出し\n\n本文\n";
        assert_eq!(section(input, "見出し"), Some("\n本文\n"));
    }

    #[test]
    fn it_section_ends_at_same_level_heading() {
        let input = "## one\n\nfirst body\n\n## two\n\nsecond body\n";
        assert_eq!(section(input, "one"), Some("\nfirst body\n\n"));
        assert_eq!(section(input, "two"), Some("\nsecond body\n"));
    }

    #[test]
    fn it_section_ends_at_higher_level_heading() {
        let input = "### sub\n\nsub body\n\n## parent\n\nparent body\n";
        assert_eq!(section(input, "sub"), Some("\nsub body\n\n"));
    }

    #[test]
    fn it_section_includes_nested_lower_level_headings() {
        let input = "## outer\n\nouter body\n\n### inner\n\ninner body\n";
        let res = section(input, "outer").unwrap();
        assert!(res.contains("outer body"));
        assert!(res.contains("### inner"));
        assert!(res.contains("inner body"));
    }

    #[test]
    fn it_section_inner_heading_lookup_returns_inner_only() {
        let input = "## outer\n\nouter body\n\n### inner\n\ninner body\n\n## next\n\nnext body\n";
        let res = section(input, "inner").unwrap();
        assert!(res.contains("inner body"));
        assert!(!res.contains("next body"));
        assert!(!res.contains("outer body"));
    }

    #[test]
    fn it_section_extends_to_eof() {
        let input = "## last\n\ntrailing body";
        assert_eq!(section(input, "last"), Some("\ntrailing body"));
    }

    #[test]
    fn it_section_duplicate_slugs_first_wins() {
        let input = "## same\n\nfirst\n\n## same\n\nsecond\n";
        assert_eq!(section(input, "same"), Some("\nfirst\n\n"));
    }

    #[test]
    fn it_section_unknown_slug() {
        let input = "## present\n\nbody\n";
        assert_eq!(section(input, "missing"), None);
    }

    #[test]
    fn it_section_empty_input() {
        assert_eq!(section("", "anything"), None);
    }

    #[test]
    fn it_section_heading_with_wikilink() {
        let input = "## [[topic]]\n\nbody\n";
        assert_eq!(section(input, "topic"), Some("\nbody\n"));
    }

    #[test]
    fn it_section_composite_document() {
        let input = "\
# top

intro

## first

f-body

### deep

deep body

## second

s-body
";
        assert_eq!(section(input, "top").unwrap().contains("intro"), true);
        let first = section(input, "first").unwrap();
        assert!(first.contains("f-body"));
        assert!(first.contains("### deep"));
        assert!(!first.contains("s-body"));
        let second = section(input, "second").unwrap();
        assert!(second.contains("s-body"));
    }
}
