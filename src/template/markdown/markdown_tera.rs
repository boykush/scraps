use once_cell::sync::Lazy;
use scraps_libs::error::{anyhow::Context, ScrapResult, ScrapsError};
use tera::Tera;

static MARKDOWN_TERA: Lazy<Tera> = Lazy::new(|| {
    #[warn(clippy::redundant_closure)]
    Tera::default()
});

pub fn init(template_dir: &str) -> ScrapResult<(Tera, tera::Context)> {
    let mut tera = Tera::new(template_dir).context(ScrapsError::PublicRender)?;
    tera.extend(&MARKDOWN_TERA).unwrap();
    let context = tera::Context::new();

    Ok((tera, context))
}
