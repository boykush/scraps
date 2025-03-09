use crate::build::model::html::HtmlMetadata;
use once_cell::sync::Lazy;
use scraps_libs::error::{anyhow::Context, ScrapsResult, ScrapsError};
use tera::Tera;
use url::Url;

static TAG_TERA: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        (
            "__builtins/base.html",
            include_str!("../builtins/base.html"),
        ),
        (
            "__builtins/macros.html",
            include_str!("../builtins/macros.html"),
        ),
        ("__builtins/tag.html", include_str!("../builtins/tag.html")),
    ])
    .unwrap();
    tera
});

pub fn base(
    base_url: &Url,
    metadata: &HtmlMetadata,
    template_dir: &str,
) -> ScrapsResult<(Tera, tera::Context)> {
    let mut tera = Tera::new(template_dir).context(ScrapsError::PublicRender)?;
    tera.extend(&TAG_TERA).unwrap();

    let mut context = tera::Context::new();
    context.insert("base_url", &base_url);
    context.insert("lang_code", &metadata.lang_code().to_string());
    context.insert("title", &metadata.title());
    context.insert("description", &metadata.description());
    context.insert("favicon", &metadata.favicon());

    Ok((tera, context))
}
