use crate::libs::error::{error::ScrapError, result::ScrapResult};
use anyhow::Context;
use once_cell::sync::Lazy;
use tera::Tera;
use url::Url;

static SCRAP_TERA: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("__builtins/base.html", include_str!("builtins/base.html")),
        ("__builtins/index.html", include_str!("builtins/index.html")),
        (
            "__builtins/macros.html",
            include_str!("builtins/macros.html"),
        ),
        ("__builtins/scrap.html", include_str!("builtins/scrap.html")),
    ])
    .unwrap();
    tera
});

pub fn init(
    site_title: &str,
    site_description: &Option<String>,
    site_favicon: &Option<Url>,
    template_dir: &str,
) -> ScrapResult<(Tera, tera::Context)> {
    let mut tera = Tera::new(template_dir).context(ScrapError::PublicRenderError)?;
    tera.extend(&SCRAP_TERA).unwrap();

    let mut context = tera::Context::new();
    context.insert("title", site_title);
    context.insert("description", site_description);
    context.insert("favicon", site_favicon);

    Ok((tera, context))
}
