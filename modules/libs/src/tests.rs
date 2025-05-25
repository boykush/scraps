use std::fs::{self, remove_file, File};
use std::io::Write;
use std::path::PathBuf;

pub trait TestResource {
    fn setup(&self);
    fn teardown(&self);
}

pub struct FileResource {
    path: PathBuf,
    parent: Option<PathBuf>,
    content: Vec<u8>,
}

impl FileResource {
    pub fn new(path: &PathBuf) -> Self {
        let parent = path.parent().map(|p| p.to_owned());
        FileResource {
            path: path.to_owned(),
            parent,
            content: Vec::new(),
        }
    }

    pub fn with_content(path: &PathBuf, content: &[u8]) -> Self {
        let parent = path.parent().map(|p| p.to_owned());
        FileResource {
            path: path.to_owned(),
            parent,
            content: content.to_vec(),
        }
    }

    pub fn run<F>(&self, init_bytes: &[u8], b: F)
    where
        F: FnOnce(),
    {
        let mut file = self.open();
        file.write_all(init_bytes).unwrap();
        b();
        self.close();
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

pub struct DirResource {
    path: PathBuf,
}

impl DirResource {
    pub fn new(path: &PathBuf) -> Self {
        DirResource {
            path: path.to_owned(),
        }
    }

    pub fn run<F>(&self, b: F)
    where
        F: FnOnce(),
    {
        self.open();
        b();
        self.close();
    }

    fn open(&self) {
        fs::create_dir_all(&self.path).unwrap()
    }

    fn close(&self) {
        fs::remove_dir_all(&self.path).unwrap()
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

impl TestResources {
    pub fn new() -> Self {
        TestResources { resources: Vec::new() }
    }
    
    pub fn add_file(&mut self, path: &PathBuf, content: &[u8]) -> &mut Self {
        self.resources.push(Box::new(FileResource::with_content(path, content)));
        self
    }
    
    pub fn add_dir(&mut self, path: &PathBuf) -> &mut Self {
        self.resources.push(Box::new(DirResource::new(path)));
        self
    }
    
    pub fn run<F>(&self, test_fn: F) where F: FnOnce() {
        for resource in &self.resources {
            resource.setup();
        }
        
        test_fn();
        
        for resource in self.resources.iter().rev() {
            resource.teardown();
        }
    }
}

#[macro_export]
macro_rules! with_test_resources {
    (files: [$(($path:expr, $content:expr)),*], dirs: [$(($dir:expr)),*], || $body:block) => {
        {
            let mut resources = $crate::tests::TestResources::new();
            $(resources.add_file($path, $content);)*
            $(resources.add_dir($dir);)*
            resources.run(|| $body);
        }
    };
}
