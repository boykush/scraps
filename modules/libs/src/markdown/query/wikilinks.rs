use comrak::{
    nodes::{NodeValue, NodeWikiLink},
    parse_document, Arena,
};

use crate::model::key::ScrapKey;

use super::common::{collect_text, options, parse_wikilink_url};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiLinkRef {
    pub ctx_path: Vec<String>,
    pub title: String,
    pub heading: Option<String>,
    pub alias: Option<String>,
}

pub fn wikilinks(text: &str) -> Vec<WikiLinkRef> {
    let arena = Arena::new();
    let opts = options();
    let root = parse_document(&arena, text, &opts);
    root.descendants()
        .filter_map(|node| match &node.data().value {
            NodeValue::WikiLink(NodeWikiLink { url }) if !url.is_empty() => {
                let (ctx_path, title, heading) = parse_wikilink_url(url);
                let label = collect_text(node);
                let display = match &heading {
                    Some(h) => format!("{}#{}", url_path(&ctx_path, &title), h),
                    None => url_path(&ctx_path, &title),
                };
                let alias = if label == display { None } else { Some(label) };
                Some(WikiLinkRef {
                    ctx_path,
                    title,
                    heading,
                    alias,
                })
            }
            _ => None,
        })
        .collect()
}

fn url_path(ctx_path: &[String], title: &str) -> String {
    if ctx_path.is_empty() {
        title.to_string()
    } else {
        format!("{}/{}", ctx_path.join("/"), title)
    }
}

impl From<&WikiLinkRef> for ScrapKey {
    fn from(w: &WikiLinkRef) -> Self {
        ScrapKey::from_path_str(&url_path(&w.ctx_path, &w.title))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn link(
        ctx_path: &[&str],
        title: &str,
        heading: Option<&str>,
        alias: Option<&str>,
    ) -> WikiLinkRef {
        WikiLinkRef {
            ctx_path: ctx_path.iter().map(|s| s.to_string()).collect(),
            title: title.to_string(),
            heading: heading.map(|s| s.to_string()),
            alias: alias.map(|s| s.to_string()),
        }
    }

    #[rstest]
    #[case::basic("[[a]]", vec![link(&[], "a", None, None)])]
    #[case::ctx_title("[[Book/Test-driven development]]",
        vec![link(&["Book"], "Test-driven development", None, None)])]
    #[case::deep_ctx("[[a/b/c]]", vec![link(&["a", "b"], "c", None, None)])]
    #[case::alias_pipe("[[Domain Driven Design|DDD]]",
        vec![link(&[], "Domain Driven Design", None, Some("DDD"))])]
    #[case::alias_special_chars("[[topic|hello, world!]]",
        vec![link(&[], "topic", None, Some("hello, world!"))])]
    #[case::heading_only("[[topic#section]]",
        vec![link(&[], "topic", Some("section"), None)])]
    #[case::ctx_heading_alias("[[Person/Eric Evans#bio|Eric]]",
        vec![link(&["Person"], "Eric Evans", Some("bio"), Some("Eric"))])]
    #[case::contain_space("[[contain space]]", vec![link(&[], "contain space", None, None)])]
    #[case::ctx_alias("[[Person/Eric Evans|Eric Evans]]",
        vec![link(&["Person"], "Eric Evans", None, Some("Eric Evans"))])]
    fn it_wikilinks_base(#[case] input: &str, #[case] expected: Vec<WikiLinkRef>) {
        assert_eq!(wikilinks(input), expected);
    }

    #[rstest]
    #[case::japanese("[[日本語タイトル]]",
        vec![link(&[], "日本語タイトル", None, None)])]
    #[case::emoji("[[🚀rocket]]", vec![link(&[], "🚀rocket", None, None)])]
    #[case::cjk_mix("[[Book/Domain駆動]]",
        vec![link(&["Book"], "Domain駆動", None, None)])]
    #[case::japanese_alias("[[Domain Driven Design|ドメイン駆動設計]]",
        vec![link(&[], "Domain Driven Design", None, Some("ドメイン駆動設計"))])]
    fn it_wikilinks_unicode(#[case] input: &str, #[case] expected: Vec<WikiLinkRef>) {
        assert_eq!(wikilinks(input), expected);
    }

    #[test]
    fn it_wikilinks_lf_joined() {
        let input = "[[a]]\n[[b]]\n[[c]]";
        let res = wikilinks(input);
        assert_eq!(res.len(), 3);
    }

    #[test]
    fn it_wikilinks_crlf_joined() {
        let input = "[[a]]\r\n[[b]]\r\n[[c]]";
        let res = wikilinks(input);
        assert_eq!(res.len(), 3);
    }

    #[test]
    fn it_wikilinks_no_trailing_newline() {
        let input = "[[a]] [[b]]";
        let res = wikilinks(input);
        assert_eq!(res.len(), 2);
    }

    #[test]
    fn it_wikilinks_blank_line_gap() {
        let input = "[[a]]\n\n\n[[b]]";
        let res = wikilinks(input);
        assert_eq!(res.len(), 2);
    }

    #[rstest]
    #[case::inline_code("`[[x]]`")]
    #[case::fenced_code("```\n[[x]]\n```")]
    #[case::indented_code("    [[x]]")]
    #[case::tilde_fence("~~~\n[[x]]\n~~~")]
    fn it_wikilinks_excludes_code(#[case] input: &str) {
        assert!(wikilinks(input).is_empty());
    }

    #[rstest]
    #[case::empty_brackets("[[]]")]
    #[case::only_open("[[only open")]
    #[case::only_close("only close]]")]
    #[case::single_brackets("[single]")]
    #[case::space_between("[ [space] ]")]
    #[case::multi_pipe("[[a|b|c]]")]
    fn it_wikilinks_invalid(#[case] input: &str) {
        assert!(wikilinks(input).is_empty());
    }

    #[test]
    fn it_wikilinks_preserves_duplicates() {
        let input = "[[a]] [[a]] [[a]] [[b]] [[b]] [[c]]";
        let result = wikilinks(input);
        assert_eq!(result.len(), 6);
        let a = link(&[], "a", None, None);
        assert_eq!(result.iter().filter(|w| **w == a).count(), 3);
    }

    #[test]
    fn it_wikilinks_composite_document() {
        let input = "\
# heading

[[plain]] and [[Book/TDD|TDD book]] in body.

```
[[ignored]]
```

`[[also ignored]]` near [[Person/Eric#bio]].
";
        let res = wikilinks(input);
        assert_eq!(res.len(), 3);
        assert_eq!(res[0], link(&[], "plain", None, None));
        assert_eq!(res[1], link(&["Book"], "TDD", None, Some("TDD book")));
        assert_eq!(res[2], link(&["Person"], "Eric", Some("bio"), None));
    }

    #[rstest]
    #[case::no_ctx(link(&[], "a", None, None), "a", None)]
    #[case::single_ctx(link(&["ctx"], "a", None, None), "a", Some("ctx"))]
    #[case::two_deep(link(&["a", "b"], "c", None, None), "c", Some("a/b"))]
    #[case::three_deep(link(&["a", "b", "c"], "d", None, None), "d", Some("a/b/c"))]
    fn it_wikilinkref_into_scrapkey(
        #[case] w: WikiLinkRef,
        #[case] expected_title: &str,
        #[case] expected_ctx: Option<&str>,
    ) {
        use crate::model::context::Ctx;
        use crate::model::title::Title;
        let key: ScrapKey = (&w).into();
        assert_eq!(Title::from(&key), expected_title.into());
        let ctx_str = Option::<Ctx>::from(&key).as_ref().map(|c| format!("{}", c));
        assert_eq!(ctx_str.as_deref(), expected_ctx);
    }
}
