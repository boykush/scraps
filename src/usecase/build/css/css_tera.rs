use crate::error::{anyhow::Context, BuildError, ScrapsResult};
use once_cell::sync::Lazy;
use tera::Tera;

use crate::usecase::build::model::color_scheme::ColorScheme;

use super::serde::color_scheme::ColorSchemeTera;

static CSS_TERA: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::default();
    crate::service::tera_filters::register(&mut tera);
    tera.add_raw_templates(vec![
        (
            "__builtins/_tokens.css",
            include_str!("builtins/_tokens.css"),
        ),
        ("__builtins/main.css", include_str!("builtins/main.css")),
    ])
    .expect("builtin templates are compiled into the binary");
    tera
});

/// Loading the glob re-parses every template, so build this once and reuse it.
pub fn tera(template_dir: &str) -> ScrapsResult<Tera> {
    let mut tera = Tera::clone(&CSS_TERA);
    tera.load_from_glob(template_dir)
        .context(BuildError::RenderCss)?;

    Ok(tera)
}

pub fn context(color_scheme: &ColorScheme) -> tera::Context {
    let mut context = tera::Context::new();
    context.insert("color_scheme", &ColorSchemeTera::new(color_scheme));

    context
}
