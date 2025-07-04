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
    use scraps_libs::tests::TestResources;

    use crate::usecase::build::model::color_scheme::ColorScheme;

    use super::*;
    use std::fs;

    #[test]
    fn test_render_main() {
        // args
        let css_metadata = &CssMetadata::new(&ColorScheme::OsSetting);

        let test_resource_path = PathBuf::from("tests/resource/build/css/render/it_render_main");
        let static_dir_path = test_resource_path.join("static");
        let public_dir_path = PathBuf::from("public");

        // static
        let template_css_path = static_dir_path.join("main.css");
        let resource_template_css_byte = ":root { color-scheme: {{ color_scheme }};}".as_bytes();

        let mut test_resources = TestResources::new();
        test_resources
            .add_file(&template_css_path, resource_template_css_byte)
            .add_dir(&public_dir_path);

        test_resources.run(|| {
            // run
            let render = CSSRender::new(&static_dir_path, &public_dir_path);
            render.render_main(css_metadata).unwrap();

            let result = fs::read_to_string(public_dir_path.join("main.css")).unwrap();
            assert_eq!(result, ":root { color-scheme: light dark;}");
        });
    }
}
