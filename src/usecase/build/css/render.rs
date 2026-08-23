use std::path::{Path, PathBuf};

use crate::error::ScrapsResult;
use crate::service::tera_render::{render_to_file, resolve_template, user_template_glob};
use tera::Tera;

use crate::usecase::build::model::css::CssMetadata;

use super::css_tera;

pub struct CSSRender {
    tera: Tera,
    output_dir_path: PathBuf,
}

impl CSSRender {
    pub fn new(static_dir_path: &Path, output_dir_path: &Path) -> ScrapsResult<CSSRender> {
        let tera = css_tera::tera(&user_template_glob(static_dir_path, "*.css"))?;

        Ok(CSSRender {
            tera,
            output_dir_path: output_dir_path.to_path_buf(),
        })
    }

    pub fn render_main(&self, css_metadata: &CssMetadata) -> ScrapsResult<()> {
        let context = css_tera::context(&css_metadata.color_scheme);
        let template_name = resolve_template(&self.tera, "main.css", "__builtins/main.css");
        let file_path = self.output_dir_path.join("main.css");
        render_to_file(&self.tera, template_name, &context, &file_path)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
    use crate::usecase::build::model::color_scheme::ColorScheme;
    use rstest::rstest;

    use super::*;
    use std::fs;

    #[rstest]
    fn test_render_main(#[from(temp_scrap_project)] project: TempScrapProject) {
        // Add static CSS template
        project.add_static_file("main.css", b":root { color-scheme: {{ color_scheme }};}");

        let css_metadata = &CssMetadata::new(&ColorScheme::OsSetting);

        // Run render
        let render = CSSRender::new(&project.static_dir, &project.output_dir).unwrap();
        render.render_main(css_metadata).unwrap();

        let result = fs::read_to_string(project.output_path("main.css")).unwrap();
        assert_eq!(result, ":root { color-scheme: light dark;}");
    }
}
