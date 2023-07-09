use std::{fs, path::PathBuf, time::UNIX_EPOCH};

use anyhow::Context;

use crate::libs::error::{error::ScrapError, result::ScrapResult};

pub fn updated_ts(path: &PathBuf) -> ScrapResult<u64> {
    let systemtime = fs::metadata(path)
        .and_then(|m| m.modified())
        .context(ScrapError::FileLoadError)?;

    systemtime
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .context(ScrapError::SystemTimeConvertError)
}
