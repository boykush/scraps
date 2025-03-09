use once_cell::sync::Lazy;
use scraps_libs::error::{anyhow::Context, ScrapsResult, ScrapsError};
use tera::Tera;

use crate::build::model::color_scheme::ColorScheme;

use super::serde::color_scheme::ColorSchemeTera;

static CSS_TERA: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::default();
    tera.add_raw_templates(vec![(
        "__builtins/main.css",
        include_str!("builtins/main.css"),
    )])
    .unwrap();
    tera
});

pub fn base(template_dir: &str, color_scheme: &ColorScheme) -> ScrapsResult<(Tera, tera::Context)> {
    let mut tera = Tera::new(template_dir).context(ScrapsError::PublicRender)?;
    tera.extend(&CSS_TERA).unwrap();

    let mut context = tera::Context::new();
    context.insert("color_scheme", &ColorSchemeTera::new(color_scheme));

    Ok((tera, context))
}
