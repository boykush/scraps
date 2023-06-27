use crate::libs::error::{error::ScrapError, result::ScrapResult};
use anyhow::Context;
use config::Config;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ScrapConfig {
    pub title: String,
    pub description: Option<String>,
}

impl ScrapConfig {
    pub fn new() -> ScrapResult<ScrapConfig> {
        let config = Config::builder()
            .add_source(config::File::with_name("Config.toml"))
            .build()
            .context(ScrapError::ConfigLoadError)?;
        config
            .try_deserialize::<ScrapConfig>()
            .context(ScrapError::ConfigLoadError)
    }
}
