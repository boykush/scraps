use crate::{
    build::{cmd::HtmlMetadata, model::sort::SortKey},
    libs::error::{error::ScrapError, result::ScrapResult},
};
use anyhow::Context;
use chrono_tz::Tz;
use once_cell::sync::Lazy;
use tera::Tera;

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
        ("__builtins/tag.html", include_str!("builtins/tag.html")),
    ])
    .unwrap();
    tera
});

pub fn init(
    timezone: &Tz,
    metadata: &HtmlMetadata,
    sort_key: &SortKey,
    template_dir: &str,
) -> ScrapResult<(Tera, tera::Context)> {
    let mut tera = Tera::new(template_dir).context(ScrapError::PublicRenderError)?;
    tera.extend(&SCRAP_TERA).unwrap();

    let mut context = tera::Context::new();
    context.insert("timezone", timezone);
    context.insert("title", &metadata.title());
    context.insert("description", &metadata.description());
    context.insert("favicon", &metadata.favicon());

    let sort_key_text = match sort_key {
        SortKey::CommittedDate => "commited date",
        SortKey::LinkedCount => "linked count",
    };
    context.insert("sort_key", sort_key_text);

    Ok((tera, context))
}
