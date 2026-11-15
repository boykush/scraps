use comrak::{Arena, nodes::NodeValue, parse_document};

use super::common::{line_col_to_byte, line_starts, options};
use super::frontmatter::mask;

/// `text` with everything that is not running prose blanked out: the
/// frontmatter block, links of every kind with their text, images, code,
/// raw HTML and bare URLs. Blanked bytes become spaces and line breaks stay,
/// so a match found in the result sits at the same byte offset in `text`.
pub fn prose(text: &str) -> String {
    let masked = mask(text);
    let arena = Arena::new();
    let root = parse_document(&arena, &masked, &options());
    let starts = line_starts(&masked);
    let mut out = masked.to_string();

    for node in root.descendants() {
        let quoted = matches!(
            node.data().value,
            NodeValue::Code(_)
                | NodeValue::CodeBlock(_)
                | NodeValue::HtmlBlock(_)
                | NodeValue::HtmlInline(_)
                | NodeValue::Link(_)
                | NodeValue::Image(_)
                | NodeValue::WikiLink(_)
        );
        if !quoted {
            continue;
        }
        let pos = node.data().sourcepos;
        let start = line_col_to_byte(&starts, pos.start.line, pos.start.column);
        // comrak's end column is inclusive.
        let end = line_col_to_byte(&starts, pos.end.line, pos.end.column) + 1;
        blank(&mut out, start, end);
    }

    for scheme in ["https://", "http://"] {
        let mut from = 0;
        while let Some(found) = out[from..].find(scheme) {
            let start = from + found;
            let end = out[start..]
                .find(char::is_whitespace)
                .map_or(out.len(), |len| start + len);
            blank(&mut out, start, end);
            from = end;
        }
    }
    out
}

/// Blank `out[start..end]`, widened to whole characters, keeping line breaks.
fn blank(out: &mut String, mut start: usize, mut end: usize) {
    end = end.min(out.len());
    while start > 0 && !out.is_char_boundary(start) {
        start -= 1;
    }
    while end < out.len() && !out.is_char_boundary(end) {
        end += 1;
    }
    if start >= end {
        return;
    }
    let spaced: String = out[start..end]
        .chars()
        .map(|c| match c {
            '\n' | '\r' => c.to_string(),
            _ => " ".repeat(c.len_utf8()),
        })
        .collect();
    out.replace_range(start..end, &spaced);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kept(text: &str) -> Vec<String> {
        prose(text).split_whitespace().map(str::to_string).collect()
    }

    #[test]
    fn it_keeps_plain_prose_as_it_is() {
        let text = "# Heading\n\nA plain sentence about Rust.\n\n- an item\n";
        assert_eq!(prose(text), text);
    }

    #[test]
    fn it_keeps_byte_offsets_and_line_breaks() {
        let text = "前の文 [[Rust]] と Rust の話。\n次の行 `Rust` です。\n";
        let out = prose(text);
        assert_eq!(out.len(), text.len());
        assert_eq!(out.matches('\n').count(), text.matches('\n').count());
        let at = text.rfind("Rust の").unwrap();
        assert_eq!(&out[at..at + 4], "Rust");
    }

    #[test]
    fn it_blanks_wiki_links_and_tags() {
        assert_eq!(
            kept("See [[Rust]], [[Lang/Rust|it]] and #[[Rust]] here."),
            vec!["See", ",", "and", "#", "here."]
        );
    }

    #[test]
    fn it_blanks_markdown_links_and_images() {
        assert_eq!(
            kept("Read [Rust book](https://example.com) or ![Rust logo](rust.png) now."),
            vec!["Read", "or", "now."]
        );
    }

    #[test]
    fn it_blanks_code() {
        let text = "Use `Rust` here.\n\n```rust\nfn rust() {}\n```\n\nAfter.\n";
        assert_eq!(kept(text), vec!["Use", "here.", "After."]);
    }

    #[test]
    fn it_blanks_raw_html_and_bare_urls() {
        let text =
            "Before <span>Rust</span> after https://example.com/Rust end.\n\n<!-- Rust -->\n";
        assert_eq!(kept(text), vec!["Before", "Rust", "after", "end."]);
    }

    #[test]
    fn it_blanks_the_frontmatter_block() {
        let text = "---\ntitle: Rust\n---\n\nAbout Rust.\n";
        assert_eq!(kept(text), vec!["About", "Rust."]);
    }
}
