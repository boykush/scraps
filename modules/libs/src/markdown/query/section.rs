use comrak::{nodes::NodeValue, parse_document, Arena};

use super::common::{collect_text, line_byte_offset, line_starts, options};

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
mod tests {
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
