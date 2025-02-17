use scraps_libs::error::{anyhow::Context, ScrapError, ScrapResult};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TemplateMetadata {
    pub title: String,
}

impl TemplateMetadata {
    pub fn new(metadata_text: &str) -> ScrapResult<TemplateMetadata> {
        toml::from_str(metadata_text).context(ScrapError::TemplateMetadataLoad)
    }
}
