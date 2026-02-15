use std::{
    fs::{self},
    path::{Path, PathBuf},
};

use anyhow::Context;
use scraps_libs::model::scrap::Scrap;

use crate::error::{ScrapsError, ScrapsResult};

pub(crate) fn to_scrap_paths(dir_path: &Path) -> ScrapsResult<Vec<PathBuf>> {
    let read_dir = fs::read_dir(dir_path).context(ScrapsError::ReadScraps)?;

    let paths = read_dir
        .map(|entry_res| {
            let entry = entry_res?;
            match entry.file_type() {
                Ok(file_type) if file_type.is_file() => {
                    let file_path = entry.path();
                    match file_path.extension() {
                        Some(ext) if ext == "md" => Ok(vec![file_path]),
                        _ => Ok(vec![]),
                    }
                }
                Ok(file_type) if file_type.is_dir() => to_scrap_paths(&entry.path()),
                res => res
                    .map(|_| vec![])
                    .context(ScrapsError::ReadScrap(entry.path())),
            }
        })
        .collect::<ScrapsResult<Vec<Vec<PathBuf>>>>()?;

    Ok(paths.into_iter().flatten().collect::<Vec<PathBuf>>())
}

pub(crate) fn to_scrap_by_path(
    scraps_dir_path: &Path,
    scrap_file_path: &Path,
) -> ScrapsResult<Scrap> {
    let file_prefix = scrap_file_path
        .file_stem()
        .ok_or(ScrapsError::ReadScrap(scrap_file_path.to_path_buf()))
        .map(|o| o.to_str())
        .and_then(|fp| fp.ok_or(ScrapsError::ReadScrap(scrap_file_path.to_path_buf())))?;
    let changed_directory_path = scrap_file_path
        .strip_prefix(scraps_dir_path)
        .context(ScrapsError::ReadScrap(scrap_file_path.to_path_buf()))?;
    let folder_name = changed_directory_path
        .parent()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty());
    let md_text = fs::read_to_string(scrap_file_path)
        .context(ScrapsError::ReadScrap(scrap_file_path.to_path_buf()))?;
    let scrap = Scrap::new(file_prefix, &folder_name, &md_text);

    Ok(scrap)
}
