use serde::Deserialize;

use crate::build::model::color_scheme::ColorScheme;

#[derive(Deserialize)]
#[serde(remote = "ColorScheme", rename_all = "snake_case")]
pub enum SerdeColorScheme {
    OsSetting,
    OnlyLight,
    OnlyDark,
}

#[derive(Deserialize, Debug)]
pub struct ColorSchemeConfig(#[serde(with = "SerdeColorScheme")] ColorScheme);

impl ColorSchemeConfig {
    pub fn into_sort_key(self) -> ColorScheme {
        self.0.clone()
    }
}