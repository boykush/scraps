use regex::Regex;

use crate::libs::markdown;

#[derive(PartialEq, Clone, Debug)]
pub struct Scrap {
    pub title: String,
    pub links: Vec<String>,
    pub html_content: String,
}

impl Scrap {
    pub fn new(title: &str, text: &str) -> Scrap {
        let re = Regex::new(r"\[\[(?P<title>[^,\s]+)\]\]").unwrap();

        let links = re
            .captures_iter(text)
            .into_iter()
            .map(|caps| caps["title"].to_string())
            .collect();

        let html_text = markdown::to_html(text);

        Scrap {
            title: title.to_string(),
            links: links,
            html_content: html_text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_new() {
        let scrap1 = Scrap::new("scrap1", "[[link1]]");
        assert_eq!(
            scrap1,
            Scrap {
                title: "scrap1".to_string(),
                links: vec!("link1".to_string()),
                html_content: "<p><a href=\"./link1.html\">link1</a></p>\n".to_string()
            }
        );

        let scrap2 = Scrap::new("scrap2", "[[link1]] [[link2]]");
        assert_eq!(
            scrap2,
            Scrap {
                title: "scrap2".to_string(),
                links: vec!("link1".to_string(), "link2".to_string()),
                html_content:
                    "<p><a href=\"./link1.html\">link1</a> <a href=\"./link2.html\">link2</a></p>\n"
                        .to_string()
            }
        )
    }
}
