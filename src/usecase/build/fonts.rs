mod builtins;

use crate::error::ScrapsResult;
use crate::service::tera_render::RenderOutput;

use builtins::BUILTINS;

/// Writes the web fonts the theme names, their stylesheet and their licenses.
/// `fonts/` owns the versions and generates the list; see
/// `mise run fonts:build`.
pub struct FontsRender {
    output: RenderOutput,
}

impl FontsRender {
    pub fn new(output: &RenderOutput) -> FontsRender {
        FontsRender {
            output: output.join("fonts"),
        }
    }

    pub fn run(&self) -> ScrapsResult<()> {
        BUILTINS
            .iter()
            .try_for_each(|(name, bytes)| self.output.write(name, bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{TempScrapProject, temp_scrap_project};
    use rstest::rstest;
    use std::fs;

    #[rstest]
    fn it_writes_every_font_under_fonts(#[from(temp_scrap_project)] project: TempScrapProject) {
        FontsRender::new(&RenderOutput::Dir(project.output_dir.clone()))
            .run()
            .unwrap();

        for (name, bytes) in BUILTINS {
            let written = fs::read(project.output_path(&format!("fonts/{name}"))).unwrap();
            assert_eq!(&written, bytes);
        }
    }

    /// A file the stylesheet names but the build does not write is a font
    /// that silently falls back in the browser.
    #[rstest]
    fn the_stylesheet_names_only_files_that_are_written() {
        let (_, css) = BUILTINS.iter().find(|(n, _)| *n == "fonts.css").unwrap();
        let css = std::str::from_utf8(css).unwrap();

        let named: Vec<&str> = css
            .split("url(./")
            .skip(1)
            .filter_map(|rest| rest.split(')').next())
            .collect();

        assert!(!named.is_empty());
        for name in named {
            assert!(BUILTINS.iter().any(|(n, _)| *n == name), "{name}");
        }
    }
}
