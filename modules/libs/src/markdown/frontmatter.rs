use comrak::{nodes::NodeValue, options::Extension, parse_document, Arena, Options};

const DELIMITER: &str = "+++";

fn options() -> Options<'static> {
    Options {
        extension: Extension {
            front_matter_delimiter: Some(DELIMITER.to_string()),
            ..Extension::default()
        },
        ..Options::default()
    }
}

pub fn get_metadata_text(text: &str) -> Option<String> {
    let arena = Arena::new();
    let opts = options();
    let root = parse_document(&arena, text, &opts);
    for child in root.children() {
        if let NodeValue::FrontMatter(fm) = &child.data().value {
            return Some(strip_delimiters(fm));
        }
    }
    None
}

pub fn ignore_metadata(text: &str) -> String {
    let arena = Arena::new();
    let opts = options();
    let root = parse_document(&arena, text, &opts);
    for child in root.children() {
        if let NodeValue::FrontMatter(fm) = &child.data().value {
            // comrak's FrontMatter content spans from the opening `+++` up to and
            // including any trailing blank lines after the closing `+++`. To match
            // pulldown-cmark's behavior of `text[range.end..].replacen("\n", "", 1)`
            // — where range.end is right after the closing delimiter — strip those
            // trailing newlines and use that length as the cut point.
            let cut = fm.trim_end_matches('\n').len();
            return text[cut..].replacen('\n', "", 1);
        }
    }
    text.to_string()
}

fn strip_delimiters(fm: &str) -> String {
    let trimmed = fm.trim_end_matches('\n');
    let after_open = trimmed
        .strip_prefix(DELIMITER)
        .map(|s| s.strip_prefix('\n').unwrap_or(s))
        .unwrap_or(trimmed);
    let body = after_open
        .strip_suffix(DELIMITER)
        .unwrap_or(after_open)
        .to_string();
    body
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::basic("+++\ntitle = \"title\"\n+++\n\n## Scrap", Some("title = \"title\"\n"))]
    #[case::multiple_fields(
        "+++\ntitle = \"title\"\ntest = \"hoge\"\n+++\n\n## Scrap",
        Some("title = \"title\"\ntest = \"hoge\"\n")
    )]
    #[case::with_trailing_newline("+++\ntitle = \"title\"\n+++\n", Some("title = \"title\"\n"))]
    #[case::no_trailing_newline("+++\ntitle = \"title\"\n+++", Some("title = \"title\"\n"))]
    #[case::unclosed_block("+++\ntitle = \"title\"\n\n\n## Scrap", None)]
    #[case::no_closing("+++\ntitle = \"title\"\n", None)]
    #[case::no_opening("title = \"title\"\n+++\n\n## Scrap", None)]
    #[case::no_opening_trailing("title = \"title\"\n+++\n", None)]
    #[case::yaml_style_with_body("---\ntitle = \"title\"\n---\n\n## Scrap", None)]
    #[case::yaml_style_trailing("---\ntitle = \"title\"\n---\n", None)]
    fn it_metadata_text(#[case] input: &str, #[case] expected: Option<&str>) {
        assert_eq!(get_metadata_text(input), expected.map(|s| s.to_string()));
    }

    #[rstest]
    #[case::yaml_style_with_body(
        "---\ntitle = \"title\"\n---\n\n## Scrap",
        "---\ntitle = \"title\"\n---\n\n## Scrap"
    )]
    #[case::yaml_style_trailing("---\ntitle = \"title\"\n---\n", "---\ntitle = \"title\"\n---\n")]
    #[case::basic("+++\ntitle = \"title\"\n+++\n\n## Scrap", "\n## Scrap")]
    #[case::multiple_fields(
        "+++\ntitle = \"title\"\ntest = \"hoge\"\n+++\n\n## Scrap",
        "\n## Scrap"
    )]
    #[case::with_trailing_newline("+++\ntitle = \"title\"\n+++\n", "")]
    #[case::no_trailing_newline("+++\ntitle = \"title\"\n+++", "")]
    #[case::unclosed_block(
        "+++\ntitle = \"title\"\n\n\n## Scrap",
        "+++\ntitle = \"title\"\n\n\n## Scrap"
    )]
    #[case::no_closing("+++\ntitle = \"title\"\n", "+++\ntitle = \"title\"\n")]
    #[case::no_opening(
        "title = \"title\"\n+++\n\n## Scrap",
        "title = \"title\"\n+++\n\n## Scrap"
    )]
    #[case::no_opening_trailing("title = \"title\"\n+++\n", "title = \"title\"\n+++\n")]
    fn it_ignore_metadata(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(ignore_metadata(input), expected.to_string());
    }
}
