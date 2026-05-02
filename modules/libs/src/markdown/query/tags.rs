use comrak::{parse_document, Arena};

use super::common::{byte_to_line, code_byte_ranges, in_code, line_starts, options};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagOccurrence {
    pub path: Vec<String>,
    pub line: usize,
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

#[cfg(test)]
mod tests {
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
