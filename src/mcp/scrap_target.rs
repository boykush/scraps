use rmcp::ErrorData;
use rmcp::model::ErrorCode;
use scraps_libs::model::context::Ctx;
use scraps_libs::model::key::ScrapKey;
use scraps_libs::model::stem::Stem;

/// The scrap a tool call names: by `key`, or by the pair tools took before
/// keys, a file stem as `title` with an optional `ctx`. The pair stays a
/// lookup by stem, so a frontmatter title never reaches a scrap.
pub fn resolve(
    key: Option<&str>,
    title: Option<&str>,
    ctx: Option<&str>,
) -> Result<ScrapKey, ErrorData> {
    match (key, title) {
        (Some(_), Some(_)) => Err(invalid("Pass either `key` or `title`, not both.")),
        (None, None) => Err(invalid(
            "Missing `key`: pass the `key` of the scrap, as results return it (the older `title` with optional `ctx` is still accepted).",
        )),
        (Some(_), None) if ctx.is_some() => Err(invalid(
            "`ctx` goes with `title` only: a `key` already contains its context.",
        )),
        (Some(key), None) => Ok(ScrapKey::from_path_str(key)),
        (None, Some(title)) => Ok(ScrapKey::new(&Stem::from(title), &ctx.map(Ctx::from))),
    }
}

fn invalid(message: &str) -> ErrorData {
    ErrorData::new(ErrorCode(-32602), message.to_string(), None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::root_key(Some("orders"), None, None, "orders")]
    #[case::key(Some("tables/orders"), None, None, "tables/orders")]
    #[case::title_alone(None, Some("orders"), None, "orders")]
    #[case::title_and_ctx(None, Some("orders"), Some("tables"), "tables/orders")]
    #[case::title_and_nested_ctx(None, Some("orders"), Some("a/b"), "a/b/orders")]
    fn it_resolves_a_key_or_the_older_pair(
        #[case] key: Option<&str>,
        #[case] title: Option<&str>,
        #[case] ctx: Option<&str>,
        #[case] expected: &str,
    ) {
        assert_eq!(
            resolve(key, title, ctx).unwrap(),
            ScrapKey::from_path_str(expected)
        );
    }

    #[test]
    fn it_reads_the_older_title_as_a_stem_never_as_a_path() {
        let key = resolve(None, Some("tables/orders"), None).unwrap();
        assert_ne!(key, ScrapKey::from_path_str("tables/orders"));
    }

    #[rstest]
    #[case::neither(None, None, None, "Missing `key`")]
    #[case::ctx_alone(None, None, Some("tables"), "Missing `key`")]
    #[case::both(Some("tables/orders"), Some("orders"), None, "not both")]
    #[case::key_and_ctx(Some("orders"), None, Some("tables"), "`ctx` goes with `title`")]
    fn it_rejects_anything_but_exactly_one_of_key_and_title(
        #[case] key: Option<&str>,
        #[case] title: Option<&str>,
        #[case] ctx: Option<&str>,
        #[case] expected: &str,
    ) {
        let error = resolve(key, title, ctx).unwrap_err();
        assert_eq!(error.code, ErrorCode(-32602));
        assert!(error.message.contains(expected), "{}", error.message);
    }
}
