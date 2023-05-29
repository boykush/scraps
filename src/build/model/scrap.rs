use regex::Regex;

#[derive(PartialEq, Clone, Debug)]
pub struct Scrap {
    pub title: String,
    pub links: Vec<String>,
    pub text: String,
}

impl Scrap {
    pub fn new(title: &str, text: &str) -> Scrap {
        let re = Regex::new(r"\[\[(?P<title>[^,\s]+)\]\]").unwrap();

        let links = re
            .captures_iter(text)
            .into_iter()
            .map(|caps| caps["title"].to_string())
            .collect();

        let replaced_link_text = re.replace_all(text, "[$title](./${title}.html)");

        Scrap {
            title: title.to_string(),
            links: links,
            text: replaced_link_text.to_string(),
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
                text: "[link1](./link1.html)".to_string()
            }
        );

        let scrap2 = Scrap::new("scrap2", "[[link1]] [[link2]]");
        assert_eq!(
            scrap2,
            Scrap {
                title: "scrap2".to_string(),
                links: vec!("link1".to_string(), "link2".to_string()),
                text: "[link1](./link1.html) [link2](./link2.html)".to_string()
            }
        )
    }
}
