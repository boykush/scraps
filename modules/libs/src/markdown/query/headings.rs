use comrak::{nodes::NodeValue, parse_document, Arena};

use super::common::{collect_text, options};

/// Structural information for a single markdown heading.
///
/// `parent` is the text label of the closest preceding heading whose level is
/// strictly lower than this heading's, mirroring how readers nest sections.
/// Top-level headings have `parent: None`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    pub level: u8,
    pub text: String,
    pub line: usize,
    pub parent: Option<String>,
}

/// Extract structured heading information from a markdown document, in
/// occurrence order.
///
/// Returns the level, plain-text label, source line, and parent label of every
/// ATX/Setext heading. Wiki-link / inline markup inside a heading is collapsed
/// to its plain-text label, mirroring how `section()` identifies headings.
pub fn headings(text: &str) -> Vec<Heading> {
    if text.is_empty() {
        return Vec::new();
    }
    let arena = Arena::new();
    let opts = options();
    let root = parse_document(&arena, text, &opts);

    let mut out: Vec<Heading> = Vec::new();
    let mut stack: Vec<(u8, String)> = Vec::new();
    for n in root.descendants() {
        if let NodeValue::Heading(h) = &n.data().value {
            let level = h.level as u8;
            let label = collect_text(n);
            let line = n.data().sourcepos.start.line;

            while matches!(stack.last(), Some((lvl, _)) if *lvl >= level) {
                stack.pop();
            }
            let parent = stack.last().map(|(_, t)| t.clone());
            stack.push((level, label.clone()));

            out.push(Heading {
                level,
                text: label,
                line,
                parent,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h(level: u8, text: &str, line: usize, parent: Option<&str>) -> Heading {
        Heading {
            level,
            text: text.to_string(),
            line,
            parent: parent.map(|s| s.to_string()),
        }
    }

    #[test]
    fn it_headings_basic() {
        let input = "# H1\n\n## H2\n\n### H3\n";
        assert_eq!(
            headings(input),
            vec![
                h(1, "H1", 1, None),
                h(2, "H2", 3, Some("H1")),
                h(3, "H3", 5, Some("H2")),
            ]
        );
    }

    #[test]
    fn it_headings_setext() {
        let input = "Title\n=====\n\nSub\n---\n";
        let res = headings(input);
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].text, "Title");
        assert_eq!(res[0].level, 1);
        assert_eq!(res[1].text, "Sub");
        assert_eq!(res[1].level, 2);
        assert_eq!(res[1].parent.as_deref(), Some("Title"));
    }

    #[test]
    fn it_headings_with_inline_markup() {
        let input = "## Hello **bold** world\n";
        let res = headings(input);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].text, "Hello bold world");
    }

    #[test]
    fn it_headings_with_wikilink() {
        let input = "## see [[topic]]\n";
        let res = headings(input);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].text, "see topic");
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
        let res = headings(input);
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].text, "same");
        assert_eq!(res[1].text, "same");
    }

    #[test]
    fn it_headings_japanese() {
        let input = "## 見出し\n\n本文\n";
        let res = headings(input);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].text, "見出し");
    }

    #[test]
    fn it_headings_parent_resets_at_higher_level() {
        let input = "# A\n\n## B\n\n### C\n\n## D\n\n# E\n\n## F\n";
        let res = headings(input);
        assert_eq!(res.len(), 6);
        assert_eq!(res[0].parent, None); // A
        assert_eq!(res[1].parent.as_deref(), Some("A")); // B under A
        assert_eq!(res[2].parent.as_deref(), Some("B")); // C under B
        assert_eq!(res[3].parent.as_deref(), Some("A")); // D back under A
        assert_eq!(res[4].parent, None); // E top-level
        assert_eq!(res[5].parent.as_deref(), Some("E")); // F under E
    }

    #[test]
    fn it_headings_skipping_levels_picks_nearest_lower() {
        // h1 then h4: parent of h4 is h1 (only lower-level seen).
        let input = "# A\n\n#### D\n";
        let res = headings(input);
        assert_eq!(res.len(), 2);
        assert_eq!(res[1].level, 4);
        assert_eq!(res[1].parent.as_deref(), Some("A"));
    }

    #[test]
    fn it_headings_line_numbers() {
        let input = "# top\n\nbody\n\n## sub\n";
        let res = headings(input);
        assert_eq!(res[0].line, 1);
        assert_eq!(res[1].line, 5);
    }
}
