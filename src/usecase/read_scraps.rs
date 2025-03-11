use std::{
    fs::{self},
    path::PathBuf,
};

use anyhow::Context;
use scraps_libs::model::scrap::Scrap;
use url::Url;

use crate::error::{ScrapsError, ScrapsResult};

pub(crate) fn to_scrap_paths(dir_path: &PathBuf) -> ScrapsResult<Vec<PathBuf>> {
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
                },
                Ok(file_type) if file_type.is_dir() => to_scrap_paths(&entry.path()),
                res => res.map(|_| vec![]).context(ScrapsError::ReadScrap(entry.path())),
            }
        })
        .collect::<ScrapsResult<Vec<Vec<PathBuf>>>>()?;

    Ok(paths.into_iter().flatten().collect::<Vec<PathBuf>>())
}

pub(crate) fn to_scrap_by_path(base_url: &Url, path: &PathBuf) -> ScrapsResult<Scrap> {
    println!("to_scrap_by_path: {:?}", path);
    let file_prefix = path
        .file_stem()
        .ok_or(ScrapsError::ReadScrap(path.clone()))
        .map(|o| o.to_str())
        .and_then(|fp| fp.ok_or(ScrapsError::ReadScrap(path.clone())))?;
    let md_text = fs::read_to_string(path).context(ScrapsError::ReadScrap(path.clone()))?;
    let scrap = Scrap::new(base_url, file_prefix, &md_text);

    Ok(scrap)
}
