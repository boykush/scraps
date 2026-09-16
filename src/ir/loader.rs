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

/// The wiki as read through the IR, before anything is written back.
/// `objects` runs parallel to `entries`; `dirty` means the file on disk no
/// longer matches what is held here.
struct LoadedWiki {
    store: IrStore,
    entries: Vec<Loaded>,
    objects: Vec<ScrapObject>,
    git_head: Option<String>,
    dirty: bool,
}

impl LoadedWiki {
    fn save(&self) {
        let scraps: Vec<Scrap> = self.entries.iter().map(|l| l.scrap.clone()).collect();
        let mut file = IrFile::new(self.objects.clone(), link_edges(&scraps));
        file.git_head = self.git_head.clone();
        if let Err(e) = self.store.save(&file) {
            tracing::warn!(
                "could not write the IR to {}: {e}",
                self.store.file_path().display()
            );
        }
    }
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
/// rest are parsed. Nothing is written here; callers save when `dirty`.
fn load(scraps_dir: &Path, exclude_dirs: &[PathBuf]) -> ScrapsResult<LoadedWiki> {
    let paths = read_scraps::to_scrap_paths(scraps_dir, exclude_dirs)?;
    let mut sources = paths
        .into_par_iter()
        .map(|path| read_source(scraps_dir, &path))
        .collect::<ScrapsResult<Vec<Source>>>()?;
    sources.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));

    let store = IrStore::new(scraps_dir);
    let stored = store.load();
    let had_file = stored.is_some();
    let git_head = stored.as_ref().and_then(|file| file.git_head.clone());
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
            let stored = known.get(&src.rel_path);
            let (scrap, object, reused) = match stored {
                Some(object) if object.hash == src.hash => {
                    let facts = object.to_facts();
                    let scrap = Scrap::from_facts(&src.title, &src.ctx, &src.text, facts);
                    (scrap, object.clone(), true)
                }
                _ => {
                    let scrap = Scrap::new(&src.title, &src.ctx, &src.text);
                    let mut object = ScrapObject::from_scrap(&src.rel_path, &src.hash, &scrap);
                    // The timestamp follows HEAD, not the working tree, so an
                    // edited source keeps the one already recorded.
                    object.commited_ts = stored.and_then(|o| o.commited_ts);
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
    let (entries, objects): (Vec<Loaded>, Vec<ScrapObject>) =
        entries.into_iter().map(|(l, o, _)| (l, o)).unzip();

    Ok(LoadedWiki {
        store,
        entries,
        objects,
        git_head,
        dirty,
    })
}

/// All scraps, `README.md` included, as the query commands see the wiki.
pub fn load_scraps(scraps_dir: &Path, exclude_dirs: &[PathBuf]) -> ScrapsResult<Vec<Scrap>> {
    let wiki = load(scraps_dir, exclude_dirs)?;
    if wiki.dirty {
        wiki.save();
    }
    Ok(wiki.entries.into_iter().map(|l| l.scrap).collect())
}

/// Scraps plus the raw root `README.md`, which the site renders on its own
/// page rather than as a scrap. With `git_command`, each scrap's last-commit
/// timestamp comes from the IR while HEAD is the commit it was recorded
/// under, and from git otherwise.
pub fn load_scraps_with_timestamps<GC: GitCommand + Send + Sync + Copy>(
    scraps_dir: &Path,
    exclude_dirs: &[PathBuf],
    git_command: Option<GC>,
) -> ScrapsResult<ScrapsWithReadme> {
    let mut wiki = load(scraps_dir, exclude_dirs)?;
    let timestamps = match git_command {
        Some(gc) => stamp(&mut wiki, gc, scraps_dir)?,
        None => vec![None; wiki.entries.len()],
    };
    if wiki.dirty {
        wiki.save();
    }

    let mut readme_text = None;
    let mut scraps_with_ts = Vec::with_capacity(wiki.entries.len());
    for (loaded, ts) in wiki.entries.into_iter().zip(timestamps) {
        if loaded.rel_path == README_PATH {
            readme_text = Some(loaded.scrap.md_text().to_string());
        } else {
            scraps_with_ts.push((loaded.scrap, ts));
        }
    }
    Ok((scraps_with_ts, readme_text))
}

/// Fill in `commited_ts` for every object: kept from the IR while HEAD is
/// the commit it was recorded under, asked of git otherwise.
fn stamp<GC: GitCommand + Send + Sync + Copy>(
    wiki: &mut LoadedWiki,
    git_command: GC,
    scraps_dir: &Path,
) -> ScrapsResult<Vec<Option<i64>>> {
    let head = head_commit(git_command, scraps_dir)?;
    let same_head = head.is_some() && head == wiki.git_head;

    let looked_up: Vec<Option<Option<i64>>> = wiki
        .entries
        .par_iter()
        .zip(wiki.objects.par_iter())
        .map(|(loaded, object)| {
            if loaded.rel_path == README_PATH || (same_head && object.commited_ts.is_some()) {
                return Ok(None);
            }
            commited_ts(git_command, &loaded.path).map(Some)
        })
        .collect::<ScrapsResult<_>>()?;

    for (object, looked_up) in wiki.objects.iter_mut().zip(&looked_up) {
        if let Some(ts) = looked_up {
            if object.commited_ts != *ts {
                object.commited_ts = *ts;
                wiki.dirty = true;
            }
        }
    }
    if wiki.git_head != head {
        wiki.git_head = head;
        wiki.dirty = true;
    }
    Ok(wiki.objects.iter().map(|o| o.commited_ts).collect())
}

fn head_commit<GC: GitCommand>(git_command: GC, scraps_dir: &Path) -> ScrapsResult<Option<String>> {
    match git_command.head_commit(scraps_dir) {
        Ok(head) => Ok(head),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(anyhow::Error::new(e).context(BuildError::GitCommitedTs)),
    }
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

    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Counts git lookups so a test can assert none happened.
    #[derive(Clone, Copy)]
    struct CountingGit {
        head: Option<&'static str>,
        calls: &'static AtomicUsize,
    }

    impl CountingGit {
        fn new(head: Option<&'static str>) -> CountingGit {
            CountingGit {
                head,
                calls: Box::leak(Box::new(AtomicUsize::new(0))),
            }
        }

        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    impl GitCommand for CountingGit {
        fn init(&self, _path: &Path) -> std::io::Result<()> {
            Ok(())
        }
        fn commited_ts(&self, _path: &Path) -> std::io::Result<Option<i64>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(Some(1_700_000_000))
        }
        fn is_git_repository(&self, _path: &Path) -> std::io::Result<bool> {
            Ok(true)
        }
        fn head_commit(&self, _path: &Path) -> std::io::Result<Option<String>> {
            Ok(self.head.map(String::from))
        }
    }

    fn two_scrap_project() -> TempScrapProject {
        let project = TempScrapProject::new();
        project
            .add_scrap("a.md", b"# A\n")
            .add_scrap("b.md", b"[[a]]\n")
            .add_scrap("README.md", b"# Readme\n");
        project
    }

    #[test]
    fn it_records_timestamps_under_the_head_they_were_read_at() {
        let project = two_scrap_project();
        let excl = exclude(&project);
        let git = CountingGit::new(Some("h1"));

        let (scraps, _) =
            load_scraps_with_timestamps(&project.project_root, &excl, Some(git)).unwrap();

        assert_eq!(git.calls(), 2, "README is not stamped");
        assert!(scraps.iter().all(|(_, ts)| *ts == Some(1_700_000_000)));
        let ir = read_ir(&project);
        assert_eq!(ir.git_head.as_deref(), Some("h1"));
        let stamped = ir.scraps.iter().filter(|o| o.commited_ts.is_some()).count();
        assert_eq!(stamped, 2);
    }

    #[test]
    fn it_reuses_timestamps_while_head_is_unchanged() {
        let project = two_scrap_project();
        let excl = exclude(&project);
        let git = CountingGit::new(Some("h1"));
        load_scraps_with_timestamps(&project.project_root, &excl, Some(git)).unwrap();

        // A query load in between must not lose what was recorded.
        load_scraps(&project.project_root, &excl).unwrap();
        // Neither must an edit: the timestamp follows HEAD, not the file.
        project.add_scrap("a.md", b"# A edited\n");
        let (scraps, _) =
            load_scraps_with_timestamps(&project.project_root, &excl, Some(git)).unwrap();

        assert_eq!(git.calls(), 2);
        assert!(scraps.iter().all(|(_, ts)| *ts == Some(1_700_000_000)));
        assert_eq!(read_ir(&project).git_head.as_deref(), Some("h1"));
    }

    #[test]
    fn it_asks_git_again_when_head_moves() {
        let project = two_scrap_project();
        let excl = exclude(&project);
        let first = CountingGit::new(Some("h1"));
        load_scraps_with_timestamps(&project.project_root, &excl, Some(first)).unwrap();

        let second = CountingGit::new(Some("h2"));
        load_scraps_with_timestamps(&project.project_root, &excl, Some(second)).unwrap();

        assert_eq!(second.calls(), 2);
        assert_eq!(read_ir(&project).git_head.as_deref(), Some("h2"));
    }

    #[test]
    fn it_never_reuses_timestamps_without_a_head() {
        let project = two_scrap_project();
        let excl = exclude(&project);
        let git = CountingGit::new(None);

        load_scraps_with_timestamps(&project.project_root, &excl, Some(git)).unwrap();
        load_scraps_with_timestamps(&project.project_root, &excl, Some(git)).unwrap();

        assert_eq!(git.calls(), 4);
        assert_eq!(read_ir(&project).git_head, None);
    }
}
