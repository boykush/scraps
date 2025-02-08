use scraps_libs::error::anyhow::Context;
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use scraps_libs::error::{ScrapError, ScrapResult};

use crate::build::model::css::CssMetadata;

pub struct CSSRender {
    static_dir_path: PathBuf,
    public_dir_path: PathBuf,
}

impl CSSRender {
    pub fn new(static_dir_path: &PathBuf, public_dir_path: &PathBuf) -> CSSRender {
        CSSRender {
            static_dir_path: static_dir_path.to_owned(),
            public_dir_path: public_dir_path.to_owned(),
        }
    }

    pub fn render_main(&self, css_metadata: &CssMetadata) -> ScrapResult<()> {
        let builtins_css = include_str!("builtins/main.css").to_string();
        let css = fs::read_to_string(self.static_dir_path.join("main.css")).unwrap_or(builtins_css);

        let mut wtr =
            File::create(self.public_dir_path.join("main.css")).context(ScrapError::FileWrite)?;
        wtr.write_all(css.as_bytes())
            .context(ScrapError::FileWrite)?;
        wtr.flush().context(ScrapError::FileWrite)
    }
}
