use std::fmt;

use super::{context::Ctx, title::Title};

#[derive(PartialEq, Clone, Debug, PartialOrd, Eq, Ord, Hash)]
pub struct ScrapKey {
    pub title: Title,
    pub ctx: Option<Ctx>,
}

impl From<Title> for ScrapKey {
    fn from(title: Title) -> Self {
        ScrapKey { title, ctx: None }
    }
}

impl fmt::Display for ScrapKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ctx) = &self.ctx {
            write!(f, "{}/{}", ctx, self.title)
        } else {
            write!(f, "{}", self.title)
        }
    }
}

impl ScrapKey {
    pub fn new(title: &Title, ctx: &Option<Ctx>) -> ScrapKey {
        ScrapKey {
            title: title.clone(),
            ctx: ctx.clone(),
        }
    }

    pub fn with_ctx(title: &Title, ctx: &Ctx) -> ScrapKey {
        ScrapKey {
            title: title.clone(),
            ctx: Some(ctx.clone()),
        }
    }

    pub fn from_path_str(path: &str) -> ScrapKey {
        let parts = path.splitn(2, "/").collect::<Vec<&str>>();
        match parts[..] {
            [title] => ScrapKey::from(Title::from(title)),
            [ctx, title] => ScrapKey::with_ctx(&title.into(), &ctx.into()),
            _ => ScrapKey::from(Title::from("")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_from_path_str() {
        let only_title_path = ScrapKey::from_path_str("ctx/title");
        assert_eq!(only_title_path.title, "title".into());
        assert_eq!(only_title_path.ctx, Some("ctx".into()));

        let with_context_path = ScrapKey::from_path_str("title");
        assert_eq!(with_context_path.title, "title".into());
        assert_eq!(with_context_path.ctx, None);

        let nested_path = ScrapKey::from_path_str("ctx/title/extra");
        assert_eq!(nested_path.title, "title/extra".into());
        assert_eq!(nested_path.ctx, Some("ctx".into()));
    }
}
