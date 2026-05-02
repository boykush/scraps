use std::{
    fs::{self},
    path::{Path, PathBuf},
};

use anyhow::Context;
use scraps_libs::model::{context::Ctx, scrap::Scrap};

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
    // Walk the path components under scraps/ to build the (possibly nested)
    // ctx. Using components() is portable across separators.
    let ctx_segments: Vec<String> = changed_directory_path
        .parent()
        .map(|p| {
            p.components()
                .map(|c| c.as_os_str().to_string_lossy().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    let ctx: Option<Ctx> = if ctx_segments.is_empty() {
        None
    } else {
        Some(Ctx::from(ctx_segments.join("/").as_str()))
    };
    let md_text = fs::read_to_string(scrap_file_path)
        .context(ScrapsError::ReadScrap(scrap_file_path.to_path_buf()))?;
    let scrap = Scrap::new(file_prefix, &ctx, &md_text);

    Ok(scrap)
}

pub(crate) fn to_all_scraps(scraps_dir_path: &Path) -> ScrapsResult<Vec<Scrap>> {
    let paths = to_scrap_paths(scraps_dir_path)?;
    paths
        .iter()
        .map(|path| to_scrap_by_path(scraps_dir_path, path))
        .collect()
}

/// Read all scraps with optional git commit timestamps, and README text separately.
/// Used by build/serve commands that need both scraps+timestamps and README.
///
/// When `git_command` is `None`, no git subprocess is spawned and every scrap's
/// `commited_ts` is returned as `None`. When `Some`, a `git not installed`
/// failure is downgraded to `None` with a warning rather than an error.
pub(crate) fn to_all_scraps_with_timestamps<
    GC: scraps_libs::git::GitCommand + Send + Sync + Copy,
>(
    scraps_dir_path: &Path,
    git_command: Option<GC>,
) -> ScrapsResult<(Vec<(Scrap, Option<i64>)>, Option<String>)> {
    use rayon::prelude::*;

    let paths = to_scrap_paths(scraps_dir_path)?;

    // Separate README.md from other scraps
    let readme_path = scraps_dir_path.join("README.md");
    let (readme_paths, scrap_paths): (Vec<_>, Vec<_>) =
        paths.into_iter().partition(|path| path == &readme_path);

    // Read README text
    let readme_text = readme_paths
        .first()
        .map(|path| fs::read_to_string(path).context(crate::error::BuildError::ReadREADMEFile))
        .transpose()?;

    // Read scraps (with git timestamps if enabled) in parallel
    let scraps_with_ts = scrap_paths
        .into_par_iter()
        .map(|path| {
            let scrap = to_scrap_by_path(scraps_dir_path, &path)?;
            let commited_ts = match git_command {
                Some(gc) => match gc.commited_ts(&path) {
                    Ok(ts) => ts,
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                        tracing::warn!(
                            "git binary not found; skipping commited_ts for {}",
                            path.display()
                        );
                        None
                    }
                    Err(e) => {
                        return Err(
                            anyhow::Error::new(e).context(crate::error::BuildError::GitCommitedTs)
                        );
                    }
                },
                None => None,
            };
            Ok((scrap, commited_ts))
        })
        .collect::<ScrapsResult<Vec<(Scrap, Option<i64>)>>>()?;

    Ok((scraps_with_ts, readme_text))
}
