use crate::build::model::html::HtmlMetadata;
use chrono_tz::Tz;
use once_cell::sync::Lazy;
use scraps_libs::error::{anyhow::Context, ScrapError, ScrapResult};
use tera::Tera;
use url::Url;

static SCRAP_TERA: Lazy<Tera> = Lazy::new(|| {
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
        (
            "__builtins/scrap.html",
            include_str!("../builtins/scrap.html"),
        ),
    ])
    .unwrap();
    tera
});

pub fn init(
    base_url: &Url,
    timezone: Tz,
    metadata: &HtmlMetadata,
    template_dir: &str,
) -> ScrapResult<(Tera, tera::Context)> {
    let mut tera = Tera::new(template_dir).context(ScrapError::PublicRender)?;
    tera.extend(&SCRAP_TERA).unwrap();

    let mut context = tera::Context::new();
    context.insert("base_url", &base_url);
    context.insert("lang_code", &metadata.lang_code().to_string());
    context.insert("timezone", &timezone);
    context.insert("title", &metadata.title());
    context.insert("description", &metadata.description());
    context.insert("favicon", &metadata.favicon());

    Ok((tera, context))
}
