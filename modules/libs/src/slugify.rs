pub fn by_dash(v: &str) -> String {
    let trimmed = v.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let lower = trimmed.to_lowercase();
    // Refer to RFC 3986 for URI encoding https://datatracker.ietf.org/doc/html/rfc3986#section-2.2
    let with_replacements = lower
        .replace(':', " colon ")
        .replace('/', " slash ")
        .replace('?', " question ")
        .replace('#', " hash ")
        .replace('[', " left-bracket ")
        .replace(']', " right-bracket ")
        .replace('@', " at ")
        .replace('!', " exclamation ")
        .replace('$', " dollar ")
        .replace('&', " and ")
        .replace('\'', " single-quote ")
        .replace('(', " left-parenthesis ")
        .replace(')', " right-parenthesis ")
        .replace('*', " asterisk ")
        .replace('+', " plus ")
        .replace(',', " comma ")
        .replace(';', " semicolon ")
        .replace('=', " equal ");

    // Replace multiple spaces with single space, then replace space with dash
    let normalized_spaces = with_replacements
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("-");

    // Clean up multiple consecutive dashes that might result from adjacent special chars
    normalized_spaces
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::lowercase("LOWER", "lower")]
    #[case::spaces("space space", "space-space")]
    #[case::lowercase_and_spaces("LOWER space", "lower-space")]
    #[case::japanese("日本語です", "日本語です")]
    #[case::existing_slugify("exists-slugify", "exists-slugify")]
    #[case::multiple_spaces("Multiple   Spaces   Here", "multiple-spaces-here")]
    #[case::leading_trailing_spaces("  Leading and Trailing  ", "leading-and-trailing")]
    #[case::special_chars_with_spaces("Hello, World!", "hello-comma-world-exclamation")]
    #[case::mixed_special_chars("Hello/Context@Test", "hello-slash-context-at-test")]
    #[case::empty("", "")]
    #[case::only_spaces("   ", "")]
    #[case::consecutive_special_chars(
        "Hello!!  @@World",
        "hello-exclamation-exclamation-at-at-world"
    )]
    fn test_by_dash(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(by_dash(input), expected);
    }
}
