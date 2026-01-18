use serde::Deserialize;

use crate::usecase::build::model::color_scheme::ColorScheme;

#[derive(Deserialize)]
#[serde(remote = "ColorScheme", rename_all = "snake_case")]
pub enum SerdeColorScheme {
    OsSetting,
    OnlyLight,
    OnlyDark,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ColorSchemeConfig(#[serde(with = "SerdeColorScheme")] ColorScheme);

impl ColorSchemeConfig {
    pub fn into_color_scheme(self) -> ColorScheme {
        self.0.clone()
    }
}
