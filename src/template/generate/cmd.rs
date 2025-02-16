use std::path::{Path, PathBuf};

use chrono_tz::Tz;
use scraps_libs::{error::ScrapResult, model::title::Title};

use crate::template::markdown::render::MarkdownRender;

pub struct GenerateCommand {
    scraps_dir_path: PathBuf,
    templates_dir_path: PathBuf,
}

impl GenerateCommand {
    pub fn new(scraps_dir_path: &Path, templates_dir_path: &Path) -> GenerateCommand {
        GenerateCommand {
            scraps_dir_path: scraps_dir_path.to_path_buf(),
            templates_dir_path: templates_dir_path.to_path_buf(),
        }
    }
    pub fn run(
        &self,
        template_name: &str,
        input_scrap_title: &Option<Title>,
        timezone: &Tz,
    ) -> ScrapResult<()> {
        let render = MarkdownRender::new(&self.scraps_dir_path, &self.templates_dir_path);

        render.render_from_template(template_name, input_scrap_title, timezone)
    }
}

#[cfg(test)]
mod tests {
    use scraps_libs::tests::{DirResource, FileResource};

    use super::*;
    use std::fs;

    #[test]
    fn it_run_has_not_input_template_title() {
        let test_resource_path =
            PathBuf::from("tests/resource/generate/cmd/it_run_has_not_input_template_title");
        let scraps_dir_path = test_resource_path.join("scraps");
        let templates_dir_path = test_resource_path.join("templates");

        // run args
        let template_name = "it_render_from_template";
        let template_title = &None;
        let timezone = chrono_tz::Asia::Tokyo;

        // template
        let template_md_path = templates_dir_path.join(format!("{}.md", template_name));
        let resource_template_md = FileResource::new(&template_md_path);

        // scraps
        let resource_scraps_dir = DirResource::new(&scraps_dir_path);
        let scraps_md_path = scraps_dir_path.join("test_title.md");

        let template_bytes =
            "+++\n[template]\ntitle = \"test_title\"\n+++\n\n{{ \"2019-09-19T15:00:00.000Z\" | date(timezone=timezone) }}".as_bytes();
        resource_scraps_dir.run(|| {
            resource_template_md.run(template_bytes, || {
                // run
                let command = GenerateCommand::new(&scraps_dir_path, &templates_dir_path);
                command
                    .run(template_name, template_title, &timezone)
                    .unwrap();

                // assert
                let result = fs::read_to_string(scraps_md_path);
                assert_eq!(
                    result.unwrap(),
                    "+++\n[template]\ntitle = \"test_title\"\n+++\n\n2019-09-20"
                )
            });
        });
    }

    #[test]
    fn it_run_has_input_template_title() {
        let test_resource_path =
            PathBuf::from("tests/resource/generate/cmd/it_run_has_input_template_title");
        let scraps_dir_path = test_resource_path.join("scraps");
        let templates_dir_path = test_resource_path.join("templates");

        // run args
        let template_name = "it_render_from_template";
        let template_title = &Some(Title::new("override_title"));
        let timezone = chrono_tz::Asia::Tokyo;

        // template
        let template_md_path = templates_dir_path.join(format!("{}.md", template_name));
        let resource_template_md = FileResource::new(&template_md_path);

        // scraps
        let resource_scraps_dir = DirResource::new(&scraps_dir_path);
        let scraps_md_path =
            scraps_dir_path.join(format!("{}.md", template_title.as_ref().unwrap()));

        let template_bytes =
            "+++\n[template]\ntitle = \"test_title\"\n+++\n\n{{ \"2019-09-19T15:00:00.000Z\" | date(timezone=timezone) }}".as_bytes();
        resource_scraps_dir.run(|| {
            resource_template_md.run(template_bytes, || {
                // run
                let command = GenerateCommand::new(&scraps_dir_path, &templates_dir_path);
                command
                    .run(template_name, template_title, &timezone)
                    .unwrap();

                // assert
                let result = fs::read_to_string(scraps_md_path);
                assert_eq!(
                    result.unwrap(),
                    "+++\n[template]\ntitle = \"test_title\"\n+++\n\n2019-09-20"
                )
            });
        });
    }
}
