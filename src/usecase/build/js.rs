use crate::error::ScrapsResult;
use crate::service::tera_render::RenderOutput;

/// The browser libraries a page loads, with their licenses. `jslibs/` owns
/// the versions and copies the files here; see `mise run jslibs:build`.
const BUILTINS: [(&str, &[u8]); 6] = [
    ("fuse.min.js", include_bytes!("js/builtins/fuse.min.js")),
    ("fuse.LICENSE", include_bytes!("js/builtins/fuse.LICENSE")),
    (
        "highlight.min.js",
        include_bytes!("js/builtins/highlight.min.js"),
    ),
    (
        "highlight.LICENSE",
        include_bytes!("js/builtins/highlight.LICENSE"),
    ),
    (
        "mermaid.min.js",
        include_bytes!("js/builtins/mermaid.min.js"),
    ),
    (
        "mermaid.LICENSE",
        include_bytes!("js/builtins/mermaid.LICENSE"),
    ),
];

pub struct JsRender {
    output: RenderOutput,
}

impl JsRender {
    pub fn new(output: &RenderOutput) -> JsRender {
        JsRender {
            output: output.join("js"),
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
    fn it_writes_every_library_under_js(#[from(temp_scrap_project)] project: TempScrapProject) {
        JsRender::new(&RenderOutput::Dir(project.output_dir.clone()))
            .run()
            .unwrap();

        for (name, bytes) in BUILTINS {
            let written = fs::read(project.output_path(&format!("js/{name}"))).unwrap();
            assert_eq!(written, bytes);
        }
    }

    #[rstest]
    fn it_writes_nothing_when_discarding() {
        assert!(JsRender::new(&RenderOutput::Discard).run().is_ok());
    }
}
