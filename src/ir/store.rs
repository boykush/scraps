use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::schema::IrFile;

pub const IR_DIR_NAME: &str = ".scraps";
pub const IR_FILE_NAME: &str = "ir.json";

// Cache Directory Tagging Specification, so backup tools skip the directory.
const CACHEDIR_TAG: &str = "Signature: 8a477f597d28d172789f06886806bc55\n\
# This file is a cache directory tag created by scraps.\n\
# For information about cache directory tags see https://bford.info/cachedir/\n";

static WRITE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Owns the `.scraps/` directory next to `.scraps.toml`. The directory
/// gitignores itself, so a wiki never has to list it.
pub struct IrStore {
    dir: PathBuf,
}

impl IrStore {
    pub fn new(wiki_root: &Path) -> IrStore {
        IrStore {
            dir: wiki_root.join(IR_DIR_NAME),
        }
    }

    pub fn file_path(&self) -> PathBuf {
        self.dir.join(IR_FILE_NAME)
    }

    /// A missing, unreadable, or outdated file reads as "no IR yet": the
    /// loader then rebuilds from source, so a bad file never fails a command.
    pub fn load(&self) -> Option<IrFile> {
        let bytes = fs::read(self.file_path()).ok()?;
        let file: IrFile = match serde_json::from_slice(&bytes) {
            Ok(file) => file,
            Err(e) => {
                tracing::warn!("ignoring unreadable IR {}: {e}", self.file_path().display());
                return None;
            }
        };
        file.is_current().then_some(file)
    }

    /// Written to a sibling temp file and renamed into place, so a reader
    /// never sees a half-written IR and concurrent writers only race whole.
    pub fn save(&self, file: &IrFile) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        self.write_markers()?;

        let sequence = WRITE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let tmp = self.dir.join(format!(
            "{IR_FILE_NAME}.{}.{sequence}.tmp",
            std::process::id()
        ));
        let bytes = serde_json::to_vec(file)?;
        fs::write(&tmp, bytes)?;
        fs::rename(&tmp, self.file_path()).inspect_err(|_| {
            let _ = fs::remove_file(&tmp);
        })
    }

    fn write_markers(&self) -> io::Result<()> {
        let gitignore = self.dir.join(".gitignore");
        if !gitignore.exists() {
            fs::write(gitignore, "*\n")?;
        }
        let tag = self.dir.join("CACHEDIR.TAG");
        if !tag.exists() {
            fs::write(tag, CACHEDIR_TAG)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::schema::FORMAT_VERSION;
    use crate::test_fixtures::TempScrapProject;

    #[test]
    fn it_round_trips_a_file() {
        let project = TempScrapProject::new();
        let store = IrStore::new(&project.project_root);
        let file = IrFile::new(vec![], vec![]);

        store.save(&file).unwrap();

        assert_eq!(store.load(), Some(file));
    }

    #[test]
    fn it_reads_a_missing_file_as_none() {
        let project = TempScrapProject::new();
        assert_eq!(IrStore::new(&project.project_root).load(), None);
    }

    #[test]
    fn it_reads_a_corrupt_file_as_none() {
        let project = TempScrapProject::new();
        let store = IrStore::new(&project.project_root);
        fs::create_dir_all(project.project_root.join(IR_DIR_NAME)).unwrap();
        fs::write(store.file_path(), b"not json").unwrap();

        assert_eq!(store.load(), None);
    }

    #[test]
    fn it_reads_another_version_as_none() {
        let project = TempScrapProject::new();
        let store = IrStore::new(&project.project_root);
        let mut file = IrFile::new(vec![], vec![]);
        file.format_version = FORMAT_VERSION + 1;
        store.save(&file).unwrap();
        assert_eq!(store.load(), None);

        let mut file = IrFile::new(vec![], vec![]);
        file.scraps_version = "0.0.0".to_string();
        store.save(&file).unwrap();
        assert_eq!(store.load(), None);
    }

    #[test]
    fn it_marks_the_directory_once() {
        let project = TempScrapProject::new();
        let store = IrStore::new(&project.project_root);
        let dir = project.project_root.join(IR_DIR_NAME);

        store.save(&IrFile::new(vec![], vec![])).unwrap();
        assert_eq!(fs::read_to_string(dir.join(".gitignore")).unwrap(), "*\n");
        assert!(
            fs::read_to_string(dir.join("CACHEDIR.TAG"))
                .unwrap()
                .starts_with("Signature: 8a477f597d28d172789f06886806bc55")
        );

        fs::write(dir.join(".gitignore"), "custom\n").unwrap();
        store.save(&IrFile::new(vec![], vec![])).unwrap();
        assert_eq!(
            fs::read_to_string(dir.join(".gitignore")).unwrap(),
            "custom\n"
        );
    }

    #[test]
    fn it_leaves_no_temp_file_behind() {
        let project = TempScrapProject::new();
        let store = IrStore::new(&project.project_root);
        store.save(&IrFile::new(vec![], vec![])).unwrap();

        let names: Vec<String> = fs::read_dir(project.project_root.join(IR_DIR_NAME))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        assert!(names.iter().all(|n| !n.ends_with(".tmp")), "{names:?}");
    }
}
