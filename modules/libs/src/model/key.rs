use std::fmt;

use super::{context::Ctx, title::Title};

#[derive(PartialEq, Clone, Debug, PartialOrd, Eq, Ord, Hash)]
pub struct ScrapKey {
    title: Title,
    ctx: Option<Ctx>,
}

impl From<Title> for ScrapKey {
    fn from(title: Title) -> Self {
        ScrapKey { title, ctx: None }
    }
}

impl fmt::Display for ScrapKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ctx) = &self.ctx {
            write!(f, "{}/{}", ctx, &self.title)
        } else {
            write!(f, "{}", &self.title)
        }
    }
}

impl From<ScrapKey> for Title {
    fn from(val: ScrapKey) -> Self {
        val.title
    }
}

impl From<&ScrapKey> for Title {
    fn from(val: &ScrapKey) -> Self {
        val.title.clone()
    }
}

impl From<ScrapKey> for Option<Ctx> {
    fn from(val: ScrapKey) -> Self {
        val.ctx
    }
}

impl From<&ScrapKey> for Option<Ctx> {
    fn from(val: &ScrapKey) -> Self {
        val.ctx.clone()
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
        assert_eq!(Title::from(&only_title_path), "title".into());
        assert_eq!(Option::<Ctx>::from(&only_title_path), Some("ctx".into()));

        let with_context_path = ScrapKey::from_path_str("title");
        assert_eq!(Title::from(&with_context_path), "title".into());
        assert_eq!(Option::<Ctx>::from(&with_context_path), None);

        let nested_path = ScrapKey::from_path_str("ctx/title/extra");
        assert_eq!(Title::from(&nested_path), "title/extra".into());
        assert_eq!(Option::<Ctx>::from(&nested_path), Some("ctx".into()));
    }

    #[test]
    fn it_into_traits() {
        let key = ScrapKey::with_ctx(&"test_title".into(), &"test_ctx".into());

        // Test From<ScrapKey> for Title
        let title: Title = key.clone().into();
        assert_eq!(title, "test_title".into());

        // Test From<&ScrapKey> for Title
        let title_ref: Title = (&key).into();
        assert_eq!(title_ref, "test_title".into());

        // Test From<ScrapKey> for Option<Ctx>
        let ctx: Option<Ctx> = key.clone().into();
        assert_eq!(ctx, Some("test_ctx".into()));

        // Test From<&ScrapKey> for Option<Ctx>
        let ctx_ref: Option<Ctx> = (&key).into();
        assert_eq!(ctx_ref, Some("test_ctx".into()));
    }
}
