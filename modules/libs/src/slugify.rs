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

    #[test]
    fn it_by_dash() {
        assert_eq!(by_dash("LOWER"), "lower".to_string());
        assert_eq!(by_dash("space space"), "space-space".to_string());
        assert_eq!(by_dash("LOWER space"), "lower-space".to_string());
        assert_eq!(by_dash("日本語です"), "日本語です".to_string());
        assert_eq!(by_dash("exists-slugify"), "exists-slugify".to_string());
    }

    #[test]
    fn test_by_dash_multiple_spaces() {
        assert_eq!(by_dash("Multiple   Spaces   Here"), "multiple-spaces-here");
    }

    #[test]
    fn test_by_dash_leading_trailing_spaces() {
        assert_eq!(by_dash("  Leading and Trailing  "), "leading-and-trailing");
    }

    #[test]
    fn test_by_dash_special_chars_with_spaces() {
        assert_eq!(by_dash("Hello, World!"), "hello-comma-world-exclamation");
    }

    #[test]
    fn test_by_dash_mixed_special_chars() {
        // "Hello/Context@Test" -> "hello-slash-context-at-test"
        // But current implementation produces: "hello-slash-context-at-test"
        assert_eq!(by_dash("Hello/Context@Test"), "hello-slash-context-at-test");
    }

    #[test]
    fn test_by_dash_empty_string() {
        assert_eq!(by_dash(""), "");
    }

    #[test]
    fn test_by_dash_only_spaces() {
        assert_eq!(by_dash("   "), "");
    }

    #[test]
    fn test_by_dash_consecutive_special_chars() {
        // "Hello!!  @@World" -> "hello-exclamation-exclamation-at-at-world"
        assert_eq!(by_dash("Hello!!  @@World"), "hello-exclamation-exclamation-at-at-world");
    }
}
