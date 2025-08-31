use std::fmt;
use url::Url;

/// A wrapper around URL that ensures proper trailing slash normalization
#[derive(Debug, Clone, PartialEq, Eq)]
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
            url.join("/")?
        };
        Ok(Self(normalized))
    }

    /// Get a reference to the underlying URL
    pub fn as_url(&self) -> &Url {
        &self.0
    }

    /// Convert to owned URL
    pub fn into_url(self) -> Url {
        self.0
    }
}

impl fmt::Display for BaseUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<BaseUrl> for Url {
    fn from(base_url: BaseUrl) -> Self {
        base_url.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_trailing_slash() {
        let url = Url::parse("https://example.com/").unwrap();
        let base_url = BaseUrl::new(url.clone()).unwrap();
        assert_eq!(base_url.as_url(), &url);
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
        let base_url = BaseUrl::new(url.clone()).unwrap();
        assert_eq!(base_url.as_url(), &url);
    }

    #[test]
    fn test_new_with_path_without_trailing_slash() {
        let url = Url::parse("https://example.com/path").unwrap();
        let base_url = BaseUrl::new(url).unwrap();
        let expected = Url::parse("https://example.com/path/").unwrap();
        assert_eq!(base_url.as_url(), &expected);
    }

    #[test]
    fn test_display() {
        let url = Url::parse("https://example.com").unwrap();
        let base_url = BaseUrl::new(url).unwrap();
        assert_eq!(base_url.to_string(), "https://example.com/");
    }

    #[test]
    fn test_into_url() {
        let url = Url::parse("https://example.com").unwrap();
        let base_url = BaseUrl::new(url).unwrap();
        let converted_url: Url = base_url.into_url();
        assert_eq!(converted_url.as_str(), "https://example.com/");
    }
}
