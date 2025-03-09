use crate::error::{anyhow::Context, ScrapsError, ScrapsResult};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TemplateMetadata {
    pub title: String,
}

impl TemplateMetadata {
    pub fn new(metadata_text: &str) -> ScrapsResult<TemplateMetadata> {
        toml::from_str(metadata_text).context(ScrapsError::TemplateMetadataLoad)
    }
}
