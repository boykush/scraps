use serde::Serialize;

use crate::usecase::build::model::color_scheme::ColorScheme;

#[derive(Serialize)]
#[serde(remote = "ColorScheme")]
enum SerializeColorScheme {
    #[serde(rename = "light dark")]
    OsSetting,

    #[serde(rename = "only light")]
    OnlyLight,

    #[serde(rename = "only dark")]
    OnlyDark,
}

#[derive(Serialize, Debug)]
pub struct ColorSchemeTera(#[serde(with = "SerializeColorScheme")] ColorScheme);

impl ColorSchemeTera {
    pub fn new(color_scheme: &ColorScheme) -> ColorSchemeTera {
        ColorSchemeTera(color_scheme.clone())
    }
}
