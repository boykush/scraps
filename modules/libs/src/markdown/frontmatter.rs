use pulldown_cmark::{Event, MetadataBlockKind, Options, Parser, Tag, TagEnd};

pub fn get_metadata_text(text: &str) -> Option<String> {
    let parser = Parser::new_ext(text, Options::all());
    let mut in_pluses_metadata = false;
    for event in parser {
        match event {
            Event::Start(Tag::MetadataBlock(MetadataBlockKind::PlusesStyle)) => {
                in_pluses_metadata = true;
            }
            Event::Text(content) if in_pluses_metadata => {
                return Some(content.to_string());
            }
            _ => {}
        }
    }
    None
}

pub fn ignore_metadata(text: &str) -> String {
    let parser = Parser::new_ext(text, Options::all());
    for (event, range) in parser.into_offset_iter() {
        if let Event::End(TagEnd::MetadataBlock(MetadataBlockKind::PlusesStyle)) = event {
            return text[range.end..].replacen("\n", "", 1);
        }
    }
    text.to_string()
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
