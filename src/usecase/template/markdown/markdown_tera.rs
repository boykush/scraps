use crate::error::{anyhow::Context, ScrapsResult, TemplateError};
use once_cell::sync::Lazy;
use tera::Tera;

static MARKDOWN_TERA: Lazy<Tera> = Lazy::new(|| {
    #[warn(clippy::redundant_closure)]
    Tera::default()
});

pub fn base(template_dir: &str) -> ScrapsResult<(Tera, tera::Context)> {
    let mut tera = Tera::new(template_dir).context(TemplateError::RenderFailure)?;
    tera.extend(&MARKDOWN_TERA).unwrap();
    let context = tera::Context::new();

    Ok((tera, context))
}
