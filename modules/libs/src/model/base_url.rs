use url::Url;

/// A wrapper around URL that ensures proper trailing slash normalization
#[derive(Debug)]
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

    #[test]
    fn test_new_with_trailing_slash() {
        let url = Url::parse("https://example.com/").unwrap();
        let expected = url.clone();
        let base_url = BaseUrl::new(url).unwrap();
        assert_eq!(base_url.as_url(), &expected);
    }

    #[test]
    fn test_new_without_trailing_slash() {
        let url = Url::parse("https://example.com").unwrap();
        let base_url = BaseUrl::new(url).unwrap();
        let expected = Url::parse("https://example.com/").unwrap();
        assert_eq!(base_url.as_url(), &expected);
    }

    #[test]
    fn test_new_with_path_and_trailing_slash() {
        let url = Url::parse("https://example.com/path/").unwrap();
        let expected = url.clone();
        let base_url = BaseUrl::new(url).unwrap();
        assert_eq!(base_url.as_url(), &expected);
    }

    #[test]
    fn test_new_with_path_without_trailing_slash() {
        let url = Url::parse("https://example.com/path").unwrap();
        let base_url = BaseUrl::new(url).unwrap();
        let expected = Url::parse("https://example.com/path/").unwrap();
        assert_eq!(base_url.as_url(), &expected);
    }
}
