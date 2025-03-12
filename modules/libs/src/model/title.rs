use std::fmt::Display;

#[derive(PartialEq, Clone, Debug, Eq, Hash, Ord, PartialOrd)]
pub struct Title(String);

impl From<&str> for Title {
    fn from(title: &str) -> Self {
        Title(title.to_string())
    }
}

impl Display for Title {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_new() {
        let title: Title = "scrap title".into();
        assert_eq!(title.0, "scrap title".to_string());
    }
}
