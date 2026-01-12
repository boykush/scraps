use url::Url;

/// A wrapper around URL that ensures proper trailing slash normalization
#[derive(Debug, Clone)]
pub struct BaseUrl(Url);

impl BaseUrl {
    /// Create a new BaseUrl with automatic trailing slash normalization
    ///
    /// # Arguments
    /// * `url` - The base URL to normalize
    ///
    /// # Returns
    /// A BaseUrl with a guaranteed trailing slash
    ///
    /// # Errors
    /// Returns an error if the URL join operation fails
    pub fn new(url: Url) -> Result<Self, url::ParseError> {
        let normalized = if url.path().ends_with('/') {
            url
        } else {
            Url::parse((url.to_string() + "/").as_str())?
        };
        Ok(Self(normalized))
    }

    /// Get a reference to the underlying URL
    pub fn as_url(&self) -> &Url {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::with_trailing_slash("https://example.com/", "https://example.com/")]
    #[case::without_trailing_slash("https://example.com", "https://example.com/")]
    #[case::path_with_trailing("https://example.com/path/", "https://example.com/path/")]
    #[case::path_without_trailing("https://example.com/path", "https://example.com/path/")]
    fn test_base_url_normalization(#[case] input: &str, #[case] expected: &str) {
        let url = Url::parse(input).unwrap();
        let base_url = BaseUrl::new(url).unwrap();
        let expected_url = Url::parse(expected).unwrap();
        assert_eq!(base_url.as_url(), &expected_url);
    }
}
