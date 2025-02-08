use super::color_scheme::ColorScheme;

pub struct CssMetadata {
    pub color_scheme: ColorScheme,
}

impl CssMetadata {
    pub fn new(color_scheme: &ColorScheme) -> Self {
        CssMetadata {
            color_scheme: color_scheme.clone(),
        }
    }
}
