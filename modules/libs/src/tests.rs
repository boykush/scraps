use std::fs::{self, remove_file, File};
use std::io::Write;
use std::path::PathBuf;

pub trait TestResource {
    fn setup(&self);
    fn teardown(&self);
}

struct FileResource {
    path: PathBuf,
    parent: Option<PathBuf>,
    content: Vec<u8>,
}

impl FileResource {
    pub fn with_content(path: &PathBuf, content: &[u8]) -> Self {
        let parent = path.parent().map(|p| p.to_owned());
        FileResource {
            path: path.to_owned(),
            parent,
            content: content.to_vec(),
        }
    }

    fn open(&self) -> File {
        let parent = self.path.parent();
        if let Some(p) = parent {
            fs::create_dir_all(p).unwrap()
        };
        File::create(&self.path).unwrap()
    }

    fn close(&self) {
        remove_file(&self.path).unwrap_or(());
        if let Some(p) = &self.parent {
            fs::remove_dir_all(p).unwrap_or(())
        };
    }
}

impl TestResource for FileResource {
    fn setup(&self) {
        let mut file = self.open();
        file.write_all(&self.content).unwrap();
    }

    fn teardown(&self) {
        self.close();
    }
}

struct DirResource {
    path: PathBuf,
}

impl DirResource {
    pub fn new(path: &PathBuf) -> Self {
        DirResource {
            path: path.to_owned(),
        }
    }

    fn open(&self) {
        fs::create_dir_all(&self.path).unwrap()
    }

    fn close(&self) {
        fs::remove_dir_all(&self.path).unwrap_or(())
    }
}

impl TestResource for DirResource {
    fn setup(&self) {
        self.open();
    }

    fn teardown(&self) {
        self.close();
    }
}

pub struct TestResources {
    resources: Vec<Box<dyn TestResource>>,
}

impl Default for TestResources {
    fn default() -> Self {
        Self::new()
    }
}

impl TestResources {
    pub fn new() -> Self {
        TestResources {
            resources: Vec::new(),
        }
    }

    pub fn add_file(&mut self, path: &PathBuf, content: &[u8]) -> &mut Self {
        self.resources
            .push(Box::new(FileResource::with_content(path, content)));
        self
    }

    pub fn add_dir(&mut self, path: &PathBuf) -> &mut Self {
        self.resources.push(Box::new(DirResource::new(path)));
        self
    }

    pub fn run<F>(&self, test_fn: F)
    where
        F: FnOnce() + std::panic::UnwindSafe,
    {
        for resource in &self.resources {
            resource.setup();
        }

        let result = std::panic::catch_unwind(|| {
            test_fn();
        });

        for resource in self.resources.iter().rev() {
            resource.teardown();
        }

        if let Err(err) = result {
            std::panic::resume_unwind(err);
        }
    }
}
