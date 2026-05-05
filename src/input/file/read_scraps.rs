use std::{
    fs::{self},
    path::{Path, PathBuf},
};

use anyhow::Context;
use scraps_libs::model::{context::Ctx, scrap::Scrap};

use crate::error::{ScrapsError, ScrapsResult};

/// Recursively walk `dir_path` collecting `*.md` files. Skips entries whose
/// name starts with `.` (so `.git/`, `.scraps.toml`, etc. never enter the
/// traversal) and any directory whose absolute path matches `exclude_dirs`
/// (callers pass the project's `static/` and configured output directory at
/// the top level).
pub(crate) fn to_scrap_paths(
    scraps_dir_path: &Path,
    exclude_dirs: &[PathBuf],
) -> ScrapsResult<Vec<PathBuf>> {
    let read_dir = fs::read_dir(scraps_dir_path).context(ScrapsError::ReadScraps)?;

    let paths = read_dir
        .map(|entry_res| {
            let entry = entry_res?;
            let entry_path = entry.path();

            if entry.file_name().to_string_lossy().starts_with('.') {
                return Ok(vec![]);
            }

            match entry.file_type() {
                Ok(file_type) if file_type.is_file() => match entry_path.extension() {
                    Some(ext) if ext == "md" => Ok(vec![entry_path]),
                    _ => Ok(vec![]),
                },
                Ok(file_type) if file_type.is_dir() => {
                    if exclude_dirs.iter().any(|p| p == &entry_path) {
                        Ok(vec![])
                    } else {
                        to_scrap_paths(&entry_path, &[])
                    }
                }
                res => res
                    .map(|_| vec![])
                    .context(ScrapsError::ReadScrap(entry_path)),
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

pub(crate) fn to_all_scraps(
    scraps_dir_path: &Path,
    exclude_dirs: &[PathBuf],
) -> ScrapsResult<Vec<Scrap>> {
    let paths = to_scrap_paths(scraps_dir_path, exclude_dirs)?;
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
    exclude_dirs: &[PathBuf],
    git_command: Option<GC>,
) -> ScrapsResult<(Vec<(Scrap, Option<i64>)>, Option<String>)> {
    use rayon::prelude::*;

    let paths = to_scrap_paths(scraps_dir_path, exclude_dirs)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::TempScrapProject;
    use std::collections::HashSet;

    fn collect_titles(scraps: &[Scrap]) -> HashSet<String> {
        scraps
            .iter()
            .map(|s| {
                let key = s.self_key();
                let title = key.title().to_string();
                match key.ctx() {
                    Some(ctx) => format!("{ctx}/{title}"),
                    None => title,
                }
            })
            .collect()
    }

    #[test]
    fn skips_static_output_and_dotfiles() {
        let project = TempScrapProject::new();
        // Top-level scrap, plus markdown under `static/` and the build output dir
        // that should be ignored, plus a dotfile directory and dotfile.
        project.add_scrap("intro.md", b"# Intro");
        project.add_static_file("style.md", b"# In static");
        std::fs::write(project.output_dir.join("page.md"), b"# In output").unwrap();
        let dot_dir = project.project_root.join(".cache");
        std::fs::create_dir_all(&dot_dir).unwrap();
        std::fs::write(dot_dir.join("note.md"), b"# In dotdir").unwrap();
        std::fs::write(project.project_root.join(".hidden.md"), b"# Hidden").unwrap();

        let exclude = vec![project.static_dir.clone(), project.output_dir.clone()];
        let scraps = to_all_scraps(&project.project_root, &exclude).unwrap();
        let titles = collect_titles(&scraps);

        assert_eq!(titles, HashSet::from(["intro".to_string()]));
    }

    #[test]
    fn collects_nested_ctx_under_project_root() {
        let project = TempScrapProject::new();
        project.add_scrap("root.md", b"# Root");
        project.add_scrap_with_context("architecture", "overview.md", b"# Overview");

        let exclude = vec![project.static_dir.clone(), project.output_dir.clone()];
        let scraps = to_all_scraps(&project.project_root, &exclude).unwrap();
        let titles = collect_titles(&scraps);

        assert_eq!(
            titles,
            HashSet::from(["root".to_string(), "architecture/overview".to_string()])
        );
    }

    #[test]
    fn readme_at_project_root_is_partitioned() {
        let project = TempScrapProject::new();
        project
            .add_scrap("README.md", b"# Readme body")
            .add_scrap("intro.md", b"# Intro");

        let exclude = vec![project.static_dir.clone(), project.output_dir.clone()];
        let (scraps_with_ts, readme) = to_all_scraps_with_timestamps::<
            scraps_libs::git::GitCommandImpl,
        >(&project.project_root, &exclude, None)
        .unwrap();

        // Only `intro.md` is a scrap; README is returned separately.
        let titles = collect_titles(
            &scraps_with_ts
                .iter()
                .map(|(s, _)| s.clone())
                .collect::<Vec<_>>(),
        );
        assert_eq!(titles, HashSet::from(["intro".to_string()]));
        assert_eq!(readme.as_deref(), Some("# Readme body"));
    }
}
