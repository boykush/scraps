use std::path::Path;

use crate::error::ScrapsResult;
use crate::service::tera_render::{RenderOutput, resolve_template, user_template_glob};
use tera::Tera;

use crate::usecase::build::model::css::CssMetadata;

use super::css_tera;

pub struct CSSRender {
    tera: Tera,
    output: RenderOutput,
}

impl CSSRender {
    pub fn new(static_dir_path: &Path, output: &RenderOutput) -> ScrapsResult<CSSRender> {
        let tera = css_tera::tera(&user_template_glob(static_dir_path, "*.css"))?;

        Ok(CSSRender {
            tera,
            output: output.clone(),
        })
    }

    /// `has_types` gates the type styles, so a wiki that uses no `type`
    /// keeps the stylesheet it had before types reached the site.
    pub fn render_main(&self, css_metadata: &CssMetadata, has_types: bool) -> ScrapsResult<()> {
        let context = css_tera::context(&css_metadata.color_scheme, has_types);
        let template_name = resolve_template(&self.tera, "main.css", "__builtins/main.css");
        self.output
            .render(&self.tera, template_name, &context, "main.css")
    }
}

#[cfg(test)]
mod tests {
    use crate::test_fixtures::{TempScrapProject, temp_scrap_project};
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
        let render = CSSRender::new(
            &project.static_dir,
            &RenderOutput::Dir(project.output_dir.clone()),
        )
        .unwrap();
        render.render_main(css_metadata, false).unwrap();

        let result = fs::read_to_string(project.output_path("main.css")).unwrap();
        assert_eq!(result, ":root { color-scheme: light dark;}");
    }

    fn builtin_main(project: &TempScrapProject, has_types: bool) -> String {
        let render = CSSRender::new(
            &project.static_dir,
            &RenderOutput::Dir(project.output_dir.clone()),
        )
        .unwrap();
        render
            .render_main(&CssMetadata::new(&ColorScheme::OsSetting), has_types)
            .unwrap();

        fs::read_to_string(project.output_path("main.css")).unwrap()
    }

    #[rstest]
    fn it_styles_types_only_when_the_wiki_has_them(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        let without = builtin_main(&project, false);
        for selector in ["p.type", "p.type-filter"] {
            assert!(!without.contains(selector), "{selector} leaked");
        }

        let with = builtin_main(&project, true);
        assert!(with.contains("div.scrap p.type"));
        assert!(with.contains("p.type-filter"));
    }
}
