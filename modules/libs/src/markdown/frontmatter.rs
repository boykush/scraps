pub fn get_metadata_text(text: &str) -> Option<String> {
    let head: Vec<&str> = text.splitn(2, "+++\n").collect();
    match head[..] {
        ["", tail] => {
            let body: Vec<&str> = tail.splitn(2, "+++").collect();
            match body[..] {
                [metadata, _] => Some(metadata.to_string()),
                _ => None,
            }
        }
        _ => None,
    }
}

pub fn ignore_metadata(text: &str) -> String {
    let head: Vec<&str> = text.splitn(2, "+++\n").collect();
    match head[..] {
        ["", tail] => {
            let body: Vec<&str> = tail.splitn(2, "+++").collect();
            match body[..] {
                [_, body] => body.replacen("\n", "", 1).to_string(),
                _ => text.to_string(),
            }
        }
        _ => text.to_string(),
    }
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
