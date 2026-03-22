use std::path::{Path, PathBuf};

use crate::error::ScrapsResult;
use crate::usecase::build::model::css::CssMetadata;
use crate::usecase::build::port::StyleWriter;

use super::css::render::CSSRender;

pub struct FileStyleWriter {
    static_dir_path: PathBuf,
    public_dir_path: PathBuf,
    css_metadata: CssMetadata,
}

impl FileStyleWriter {
    pub fn new(static_dir_path: &Path, public_dir_path: &Path, css_metadata: CssMetadata) -> Self {
        Self {
            static_dir_path: static_dir_path.to_path_buf(),
            public_dir_path: public_dir_path.to_path_buf(),
            css_metadata,
        }
    }
}

impl StyleWriter for FileStyleWriter {
    fn write(&self) -> ScrapsResult<()> {
        let render = CSSRender::new(&self.static_dir_path, &self.public_dir_path);
        render.render_main(&self.css_metadata)
    }
}
