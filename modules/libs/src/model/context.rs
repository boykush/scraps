use std::fmt::Display;

#[derive(PartialEq, Clone, Debug)]
pub struct Ctx(String);

impl From<&str> for Ctx {
    fn from(str: &str) -> Self {
        Ctx(str.to_string())
    }
}

impl Display for Ctx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

