use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Context;
use rayon::prelude::*;
use scraps_libs::git::GitCommand;
use scraps_libs::model::{context::Ctx, scrap::Scrap};

use crate::error::{BuildError, ScrapsError, ScrapsResult};
use crate::input::file::read_scraps::{self, ScrapsWithReadme};

use super::hash::content_hash;
use super::link_table::link_edges;
use super::schema::{IrFile, ScrapObject};
use super::store::IrStore;

const README_PATH: &str = "README.md";

struct Source {
    path: PathBuf,
    rel_path: String,
    title: String,
    ctx: Option<Ctx>,
    text: String,
    hash: String,
}

struct Loaded {
    scrap: Scrap,
    path: PathBuf,
    rel_path: String,
}

fn read_source(scraps_dir: &Path, path: &Path) -> ScrapsResult<Source> {
    let rel_path = read_scraps::relative_path(scraps_dir, path)?;
    let (title, ctx) = read_scraps::scrap_identity(scraps_dir, path)?;
    let text = std::fs::read_to_string(path).context(ScrapsError::ReadScrap(path.to_path_buf()))?;
    let hash = content_hash(text.as_bytes());
    Ok(Source {
        path: path.to_path_buf(),
        rel_path,
        title,
        ctx,
        text,
        hash,
    })
}

/// Every scrap of the wiki in path order, routed through the on-disk IR:
/// a source whose hash the IR knows is rebuilt from its stored facts, the
/// rest are parsed, and the IR is rewritten when anything changed.
fn load(scraps_dir: &Path, exclude_dirs: &[PathBuf]) -> ScrapsResult<Vec<Loaded>> {
    let paths = read_scraps::to_scrap_paths(scraps_dir, exclude_dirs)?;
    let mut sources = paths
        .into_par_iter()
        .map(|path| read_source(scraps_dir, &path))
        .collect::<ScrapsResult<Vec<Source>>>()?;
    sources.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));

    let store = IrStore::new(scraps_dir);
    let stored = store.load();
    let had_file = stored.is_some();
    let known: HashMap<String, ScrapObject> = stored
        .map(|file| {
            file.scraps
                .into_iter()
                .map(|o| (o.path.clone(), o))
                .collect()
        })
        .unwrap_or_default();

    let entries: Vec<(Loaded, ScrapObject, bool)> = sources
        .into_par_iter()
        .map(|src| {
            let (scrap, object, reused) = match known.get(&src.rel_path) {
                Some(object) if object.hash == src.hash => {
                    let facts = object.to_facts();
                    let scrap = Scrap::from_facts(&src.title, &src.ctx, &src.text, facts);
                    (scrap, object.clone(), true)
                }
                _ => {
                    let scrap = Scrap::new(&src.title, &src.ctx, &src.text);
                    let object = ScrapObject::from_scrap(&src.rel_path, &src.hash, &scrap);
                    (scrap, object, false)
                }
            };
            let loaded = Loaded {
                scrap,
                path: src.path,
                rel_path: src.rel_path,
            };
            (loaded, object, reused)
        })
        .collect();

    let reused = entries.iter().filter(|(_, _, reused)| *reused).count();
    let dirty = !had_file || reused != entries.len() || known.len() != entries.len();
    let (loaded, objects): (Vec<Loaded>, Vec<ScrapObject>) =
        entries.into_iter().map(|(l, o, _)| (l, o)).unzip();

    if dirty {
        let scraps: Vec<Scrap> = loaded.iter().map(|l| l.scrap.clone()).collect();
        let file = IrFile::new(objects, link_edges(&scraps));
        if let Err(e) = store.save(&file) {
            tracing::warn!(
                "could not write the IR to {}: {e}",
                store.file_path().display()
            );
        }
    }

    Ok(loaded)
}

/// All scraps, `README.md` included, as the query commands see the wiki.
pub fn load_scraps(scraps_dir: &Path, exclude_dirs: &[PathBuf]) -> ScrapsResult<Vec<Scrap>> {
    Ok(load(scraps_dir, exclude_dirs)?
        .into_iter()
        .map(|l| l.scrap)
        .collect())
}

/// Scraps plus the raw root `README.md`, which the site renders on its own
/// page rather than as a scrap. Commit timestamps are looked up per scrap
/// when `git_command` is given, as `build --git` asks for.
pub fn load_scraps_with_timestamps<GC: GitCommand + Send + Sync + Copy>(
    scraps_dir: &Path,
    exclude_dirs: &[PathBuf],
    git_command: Option<GC>,
) -> ScrapsResult<ScrapsWithReadme> {
    let (readme, scraps): (Vec<Loaded>, Vec<Loaded>) = load(scraps_dir, exclude_dirs)?
        .into_iter()
        .partition(|l| l.rel_path == README_PATH);
    let readme_text = readme
        .into_iter()
        .next()
        .map(|l| l.scrap.md_text().to_string());

    let scraps_with_ts = scraps
        .into_par_iter()
        .map(|loaded| {
            let commited_ts = match git_command {
                Some(gc) => commited_ts(gc, &loaded.path)?,
                None => None,
            };
            Ok((loaded.scrap, commited_ts))
        })
        .collect::<ScrapsResult<Vec<_>>>()?;

    Ok((scraps_with_ts, readme_text))
}

/// A `git not installed` failure is downgraded to `None` with a warning
/// rather than an error, so `--git` degrades instead of failing the build.
fn commited_ts<GC: GitCommand>(git_command: GC, path: &Path) -> ScrapsResult<Option<i64>> {
    match git_command.commited_ts(path) {
        Ok(ts) => Ok(ts),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::warn!(
                "git binary not found; skipping commited_ts for {}",
                path.display()
            );
            Ok(None)
        }
        Err(e) => Err(anyhow::Error::new(e).context(BuildError::GitCommitedTs)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::schema::FORMAT_VERSION;
    use crate::ir::store::{IR_DIR_NAME, IR_FILE_NAME};
    use crate::test_fixtures::TempScrapProject;
    use scraps_libs::git::GitCommandImpl;
    use std::fs;

    fn exclude(project: &TempScrapProject) -> Vec<PathBuf> {
        vec![project.static_dir.clone(), project.output_dir.clone()]
    }

    fn ir_path(project: &TempScrapProject) -> PathBuf {
        project.project_root.join(IR_DIR_NAME).join(IR_FILE_NAME)
    }

    fn read_ir(project: &TempScrapProject) -> IrFile {
        serde_json::from_slice(&fs::read(ir_path(project)).unwrap()).unwrap()
    }

    fn write_ir(project: &TempScrapProject, file: &IrFile) {
        fs::write(ir_path(project), serde_json::to_vec(file).unwrap()).unwrap();
    }

    #[test]
    fn it_writes_the_ir_on_first_load() {
        let project = TempScrapProject::new();
        project
            .add_scrap("b.md", b"# B\n\n[[a]] and [[missing]]")
            .add_scrap("a.md", b"# A\n");

        let scraps = load_scraps(&project.project_root, &exclude(&project)).unwrap();

        let titles: Vec<String> = scraps.iter().map(|s| s.title().to_string()).collect();
        assert_eq!(titles, vec!["a", "b"]);
        let ir = read_ir(&project);
        assert_eq!(ir.format_version, FORMAT_VERSION);
        let paths: Vec<&str> = ir.scraps.iter().map(|o| o.path.as_str()).collect();
        assert_eq!(paths, vec!["a.md", "b.md"]);
        assert_eq!(ir.links.len(), 2);
        assert!(ir.links.iter().any(|e| e.to.title == "a" && e.resolved));
        assert!(ir
            .links
            .iter()
            .any(|e| e.to.title == "missing" && !e.resolved));
        let dir = project.project_root.join(IR_DIR_NAME);
        assert_eq!(fs::read_to_string(dir.join(".gitignore")).unwrap(), "*\n");
    }

    #[test]
    fn it_matches_a_fresh_parse_cold_and_warm() {
        let project = TempScrapProject::new();
        project
            .add_scrap(
                "a.md",
                b"# A\n\n[[Ctx/b#Sec]] #[[t]] ![[b]]\n\n- [ ] task\n\n```sh\nls\n```\n",
            )
            .add_scrap_with_context("Ctx", "b.md", b"---\nk: v\n---\n\n## Sec\n");
        let excl = exclude(&project);
        let mut expected = read_scraps::to_all_scraps(&project.project_root, &excl).unwrap();
        expected.sort_by_key(Scrap::self_key);

        let mut cold = load_scraps(&project.project_root, &excl).unwrap();
        cold.sort_by_key(Scrap::self_key);
        let mut warm = load_scraps(&project.project_root, &excl).unwrap();
        warm.sort_by_key(Scrap::self_key);

        assert_eq!(cold, expected);
        assert_eq!(warm, expected);
    }

    #[test]
    fn it_reuses_stored_facts_for_an_unchanged_source() {
        let project = TempScrapProject::new();
        project.add_scrap("a.md", b"# Real\n");
        let excl = exclude(&project);
        load_scraps(&project.project_root, &excl).unwrap();
        let mut ir = read_ir(&project);
        ir.scraps[0].headings[0].text = "Ghost".to_string();
        write_ir(&project, &ir);

        let scraps = load_scraps(&project.project_root, &excl).unwrap();

        assert_eq!(scraps[0].headings()[0].text, "Ghost");
        assert_eq!(read_ir(&project).scraps[0].headings[0].text, "Ghost");
    }

    #[test]
    fn it_reparses_a_changed_source() {
        let project = TempScrapProject::new();
        project.add_scrap("a.md", b"# Before\n");
        let excl = exclude(&project);
        load_scraps(&project.project_root, &excl).unwrap();
        let before = read_ir(&project).scraps[0].hash.clone();

        project.add_scrap("a.md", b"# After\n");
        let scraps = load_scraps(&project.project_root, &excl).unwrap();

        assert_eq!(scraps[0].headings()[0].text, "After");
        let ir = read_ir(&project);
        assert_ne!(ir.scraps[0].hash, before);
        assert_eq!(ir.scraps[0].headings[0].text, "After");
    }

    #[test]
    fn it_drops_a_removed_source() {
        let project = TempScrapProject::new();
        project
            .add_scrap("a.md", b"# A\n")
            .add_scrap("b.md", b"[[a]]\n");
        let excl = exclude(&project);
        load_scraps(&project.project_root, &excl).unwrap();

        fs::remove_file(project.project_root.join("b.md")).unwrap();
        let scraps = load_scraps(&project.project_root, &excl).unwrap();

        assert_eq!(scraps.len(), 1);
        let ir = read_ir(&project);
        assert_eq!(ir.scraps.len(), 1);
        assert!(ir.links.is_empty());
    }

    #[test]
    fn it_rebuilds_from_a_corrupt_file() {
        let project = TempScrapProject::new();
        project.add_scrap("a.md", b"# A\n");
        let excl = exclude(&project);
        load_scraps(&project.project_root, &excl).unwrap();
        fs::write(ir_path(&project), b"not json").unwrap();

        let scraps = load_scraps(&project.project_root, &excl).unwrap();

        assert_eq!(scraps[0].headings()[0].text, "A");
        assert_eq!(read_ir(&project).scraps.len(), 1);
    }

    #[test]
    fn it_discards_a_file_from_another_version() {
        let project = TempScrapProject::new();
        project.add_scrap("a.md", b"# Real\n");
        let excl = exclude(&project);
        load_scraps(&project.project_root, &excl).unwrap();
        let mut ir = read_ir(&project);
        ir.format_version = FORMAT_VERSION + 1;
        ir.scraps[0].headings[0].text = "Ghost".to_string();
        write_ir(&project, &ir);

        let scraps = load_scraps(&project.project_root, &excl).unwrap();

        assert_eq!(scraps[0].headings()[0].text, "Real");
        assert_eq!(read_ir(&project).format_version, FORMAT_VERSION);
    }

    #[test]
    fn it_partitions_the_root_readme_only_for_the_build() {
        let project = TempScrapProject::new();
        project
            .add_scrap("README.md", b"# Readme body")
            .add_scrap("a.md", b"# A\n");
        let excl = exclude(&project);

        let all = load_scraps(&project.project_root, &excl).unwrap();
        assert_eq!(all.len(), 2);

        let (scraps, readme) =
            load_scraps_with_timestamps::<GitCommandImpl>(&project.project_root, &excl, None)
                .unwrap();
        assert_eq!(scraps.len(), 1);
        assert_eq!(scraps[0].0.title().to_string(), "a");
        assert_eq!(scraps[0].1, None);
        assert_eq!(readme.as_deref(), Some("# Readme body"));
    }

    #[test]
    fn it_skips_the_ir_directory_itself() {
        let project = TempScrapProject::new();
        project.add_scrap("a.md", b"# A\n");
        let excl = exclude(&project);
        load_scraps(&project.project_root, &excl).unwrap();
        fs::write(
            project.project_root.join(IR_DIR_NAME).join("note.md"),
            b"# Not a scrap",
        )
        .unwrap();

        let scraps = load_scraps(&project.project_root, &excl).unwrap();

        assert_eq!(scraps.len(), 1);
    }
}
