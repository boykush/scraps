use super::error::{anyhow::Context, ScrapError, ScrapResult};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ScrapMetadata {
    pub template: Option<TemplateMetadata>,
}

impl ScrapMetadata {
    pub fn new(metadata_text: &str) -> ScrapResult<ScrapMetadata> {
        toml::from_str(metadata_text).context(ScrapError::ScrapMetadataLoad)
    }
}

#[derive(Deserialize)]
pub struct TemplateMetadata {
    pub title: String,
}
