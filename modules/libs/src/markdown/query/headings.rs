use comrak::{nodes::NodeValue, parse_document, Arena};

use super::common::{collect_text, options};

/// Extract heading text strings from a markdown document, in occurrence order.
///
/// Returns the visible label of every ATX/Setext heading. Levels and structural
/// information are not preserved — callers that need them should parse the
/// document themselves. Wiki-link / inline markup inside a heading is collapsed
/// to its plain-text label, mirroring how `section()` identifies headings.
pub fn headings(text: &str) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }
    let arena = Arena::new();
    let opts = options();
    let root = parse_document(&arena, text, &opts);

    let mut out = Vec::new();
    for n in root.descendants() {
        if let NodeValue::Heading(_) = &n.data().value {
            out.push(collect_text(n));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_headings_basic() {
        let input = "# H1\n\n## H2\n\n### H3\n";
        assert_eq!(headings(input), vec!["H1", "H2", "H3"]);
    }

    #[test]
    fn it_headings_setext() {
        let input = "Title\n=====\n\nSub\n---\n";
        assert_eq!(headings(input), vec!["Title", "Sub"]);
    }

    #[test]
    fn it_headings_with_inline_markup() {
        let input = "## Hello **bold** world\n";
        assert_eq!(headings(input), vec!["Hello bold world"]);
    }

    #[test]
    fn it_headings_with_wikilink() {
        let input = "## see [[topic]]\n";
        assert_eq!(headings(input), vec!["see topic"]);
    }

    #[test]
    fn it_headings_empty_input() {
        assert!(headings("").is_empty());
    }

    #[test]
    fn it_headings_no_headings() {
        let input = "just a paragraph\n\nanother paragraph\n";
        assert!(headings(input).is_empty());
    }

    #[test]
    fn it_headings_preserves_order_and_duplicates() {
        let input = "## same\n\nbody\n\n## same\n\nmore\n";
        assert_eq!(headings(input), vec!["same", "same"]);
    }

    #[test]
    fn it_headings_japanese() {
        let input = "## 見出し\n\n本文\n";
        assert_eq!(headings(input), vec!["見出し"]);
    }
}
