use crate::error::anyhow::Context;
use std::{fs::File, path::PathBuf};

use crate::error::{BuildError, ScrapsResult};

use crate::usecase::build::model::css::CssMetadata;

use super::css_tera;

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

    pub fn render_main(&self, css_metadata: &CssMetadata) -> ScrapsResult<()> {
        let (tera, context) = css_tera::base(
            self.static_dir_path.join("*.css").to_str().unwrap(),
            &css_metadata.color_scheme,
        )?;
        let template_name = if tera.get_template_names().any(|t| t == "main.css") {
            "main.css"
        } else {
            "__builtins/main.css"
        };
        let file_path = &self.public_dir_path.join("main.css");
        let wtr = File::create(file_path).context(BuildError::WriteFailure(file_path.clone()))?;
        tera.render_to(template_name, &context, wtr)
            .context(BuildError::WriteFailure(file_path.clone()))
    }
}

#[cfg(test)]
mod tests {
    use crate::test_fixtures::TempScrapProject;
    use crate::usecase::build::model::color_scheme::ColorScheme;

    use super::*;
    use std::fs;

    #[test]
    fn test_render_main() {
        let project = TempScrapProject::new();

        // Add static CSS template
        project.add_static_file("main.css", b":root { color-scheme: {{ color_scheme }};}");

        let css_metadata = &CssMetadata::new(&ColorScheme::OsSetting);

        // Run render
        let render = CSSRender::new(&project.static_dir, &project.public_dir);
        render.render_main(css_metadata).unwrap();

        let result = fs::read_to_string(project.public_path("main.css")).unwrap();
        assert_eq!(result, ":root { color-scheme: light dark;}");
    }
}
