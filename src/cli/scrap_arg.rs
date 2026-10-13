use scraps_libs::model::key::ScrapKey;

/// The scrap a command's positional argument names. Alone it is a key; with
/// `--ctx` it is the stem within that context, the form commands took before
/// they accepted keys. A stem holds no `/`, so the two never disagree.
pub fn scrap_key(positional: &str, ctx: Option<&str>) -> ScrapKey {
    match ctx {
        Some(ctx) => ScrapKey::from_path_str(&format!("{ctx}/{positional}")),
        None => ScrapKey::from_path_str(positional),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::root_key("orders", None, "orders")]
    #[case::key_with_context("tables/orders", None, "tables/orders")]
    #[case::nested_key("a/b/orders", None, "a/b/orders")]
    #[case::stem_and_ctx("orders", Some("tables"), "tables/orders")]
    #[case::stem_and_nested_ctx("orders", Some("a/b"), "a/b/orders")]
    #[case::spaces_kept("Getting Started", Some("Guide"), "Guide/Getting Started")]
    fn it_reads_a_key_or_a_stem_within_a_context(
        #[case] positional: &str,
        #[case] ctx: Option<&str>,
        #[case] expected: &str,
    ) {
        assert_eq!(
            scrap_key(positional, ctx),
            ScrapKey::from_path_str(expected)
        );
    }
}
