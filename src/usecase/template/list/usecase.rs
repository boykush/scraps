use std::path::{Path, PathBuf};

use crate::error::{anyhow::Ok, ScrapsResult};

use crate::usecase::template::markdown::markdown_tera;

pub struct ListUsecase {
    templates_dir_path: PathBuf,
}

impl ListUsecase {
    pub fn new(templates_dir_path: &Path) -> ListUsecase {
        ListUsecase {
            templates_dir_path: templates_dir_path.to_path_buf(),
        }
    }
    pub fn execute(&self) -> ScrapsResult<Vec<String>> {
        let (markdown_tera, _) =
            markdown_tera::base(self.templates_dir_path.join("*.md").to_str().unwrap())?;
        let template_names = markdown_tera
            .get_template_names()
            .map(|s| {
                Path::new(s)
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or(s)
                    .to_string()
            })
            .collect();
        Ok(template_names)
    }
}
