use comrak::{parse_document, Arena};

use super::common::{
    byte_to_line, code_byte_ranges, in_code, line_starts, options, parse_wikilink_url,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbedRef {
    pub ctx_path: Vec<String>,
    pub title: String,
    pub heading: Option<String>,
    pub line: usize,
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

#[cfg(test)]
mod tests {
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
