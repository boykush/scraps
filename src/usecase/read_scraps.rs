use std::{
    fs::{self, DirEntry},
    path::PathBuf,
};

use anyhow::{bail, Context};
use scraps_libs::model::scrap::Scrap;
use url::Url;

use crate::error::{ScrapsError, ScrapsResult};

pub(crate) fn to_path_by_dir_entry(dir_entry: &DirEntry) -> ScrapsResult<Option<PathBuf>> {
    if let Ok(file_type) = dir_entry.file_type() {
        if file_type.is_dir() {
            bail!(ScrapsError::ReadScrap(dir_entry.path()))
        }
    };
    if dir_entry.path().extension() == Some("md".as_ref()) {
        Ok(Some(dir_entry.path()))
    } else {
        Ok(None)
    }
}

pub(crate) fn to_scrap_by_path(base_url: &Url, path: &PathBuf) -> ScrapsResult<Scrap> {
    let file_prefix = path
        .file_stem()
        .ok_or(ScrapsError::ReadScrap(path.clone()))
        .map(|o| o.to_str())
        .and_then(|fp| fp.ok_or(ScrapsError::ReadScrap(path.clone())))?;
    let md_text = fs::read_to_string(path).context(ScrapsError::ReadScrap(path.clone()))?;
    let scrap = Scrap::new(base_url, file_prefix, &md_text);

    Ok(scrap)
}
