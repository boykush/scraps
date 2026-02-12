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

    #[test]
    fn it_metadata_text() {
        assert_eq!(
            get_metadata_text("+++\ntitle = \"title\"\n+++\n\n## Scrap"),
            Some("title = \"title\"\n".to_string())
        );
        assert_eq!(
            get_metadata_text("+++\ntitle = \"title\"\ntest = \"hoge\"\n+++\n\n## Scrap"),
            Some("title = \"title\"\ntest = \"hoge\"\n".to_string())
        );
        assert_eq!(
            get_metadata_text("+++\ntitle = \"title\"\n+++\n"),
            Some("title = \"title\"\n".to_string())
        );
        assert_eq!(
            get_metadata_text("+++\ntitle = \"title\"\n+++"),
            Some("title = \"title\"\n".to_string())
        );
        assert_eq!(
            get_metadata_text("+++\ntitle = \"title\"\n\n\n## Scrap"),
            None
        );
        assert_eq!(get_metadata_text("+++\ntitle = \"title\"\n"), None);
        assert_eq!(
            get_metadata_text("title = \"title\"\n+++\n\n## Scrap"),
            None
        );
        assert_eq!(get_metadata_text("title = \"title\"\n+++\n"), None);
    }

    #[test]
    fn it_metadata_text_ignores_yaml_style() {
        assert_eq!(
            get_metadata_text("---\ntitle = \"title\"\n---\n\n## Scrap"),
            None
        );
        assert_eq!(get_metadata_text("---\ntitle = \"title\"\n---\n"), None);
    }

    #[test]
    fn it_ignore_metadata_preserves_yaml_style() {
        assert_eq!(
            ignore_metadata("---\ntitle = \"title\"\n---\n\n## Scrap"),
            "---\ntitle = \"title\"\n---\n\n## Scrap".to_string()
        );
        assert_eq!(
            ignore_metadata("---\ntitle = \"title\"\n---\n"),
            "---\ntitle = \"title\"\n---\n".to_string()
        );
    }

    #[test]
    fn it_ignore_metadata() {
        assert_eq!(
            ignore_metadata("+++\ntitle = \"title\"\n+++\n\n## Scrap"),
            "\n## Scrap".to_string()
        );
        assert_eq!(
            ignore_metadata("+++\ntitle = \"title\"\ntest = \"hoge\"\n+++\n\n## Scrap"),
            "\n## Scrap".to_string()
        );
        assert_eq!(
            ignore_metadata("+++\ntitle = \"title\"\n+++\n"),
            "".to_string()
        );
        assert_eq!(
            ignore_metadata("+++\ntitle = \"title\"\n+++"),
            "".to_string()
        );
        assert_eq!(
            ignore_metadata("+++\ntitle = \"title\"\n\n\n## Scrap"),
            "+++\ntitle = \"title\"\n\n\n## Scrap".to_string()
        );
        assert_eq!(
            ignore_metadata("+++\ntitle = \"title\"\n"),
            "+++\ntitle = \"title\"\n".to_string()
        );
        assert_eq!(
            ignore_metadata("title = \"title\"\n+++\n\n## Scrap"),
            "title = \"title\"\n+++\n\n## Scrap".to_string()
        );
        assert_eq!(
            ignore_metadata("title = \"title\"\n+++\n"),
            "title = \"title\"\n+++\n".to_string()
        );
    }
}
