use serde::Serialize;

#[derive(Serialize)]
pub enum SerdeColorScheme {
    OsSetting,
    OnlyLight,
    OnlyDark,
}