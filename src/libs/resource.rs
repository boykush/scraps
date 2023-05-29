#[cfg(test)]
pub mod tests {
    use std::fs::{self, remove_file, File};
    use std::io::Write;
    use std::path::PathBuf;

    pub struct FileResource {
        path: PathBuf,
        parent: Option<PathBuf>,
    }

    impl FileResource {
        pub fn new(path: &PathBuf) -> Self {
            let parent = path.parent().map(|p| p.to_owned());
            FileResource {
                path: path.to_owned(),
                parent: parent,
            }
        }

        pub fn run<F>(&self, init_bytes: &[u8], b: F)
        where
            F: FnOnce(),
        {
            let mut file = self.open();
            file.write(init_bytes).unwrap();
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

        fn close(&self) -> () {
            remove_file(&self.path).unwrap_or(());
            if let Some(p) = &self.parent {
                fs::remove_dir_all(p).unwrap_or(())
            };
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

        fn open(&self) -> () {
            fs::create_dir_all(&self.path).unwrap()
        }

        fn close(&self) -> () {
            fs::remove_dir_all(&self.path).unwrap()
        }
    }
}
