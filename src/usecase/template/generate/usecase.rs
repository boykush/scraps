use std::path::{Path, PathBuf};

use crate::error::ScrapsResult;
use chrono_tz::Tz;
use scraps_libs::model::title::Title;

use crate::usecase::template::markdown::render::MarkdownRender;

pub struct GenerateUsecase {
    scraps_dir_path: PathBuf,
    templates_dir_path: PathBuf,
}

impl GenerateUsecase {
    pub fn new(scraps_dir_path: &Path, templates_dir_path: &Path) -> GenerateUsecase {
        GenerateUsecase {
            scraps_dir_path: scraps_dir_path.to_path_buf(),
            templates_dir_path: templates_dir_path.to_path_buf(),
        }
    }
    pub fn execute(
        &self,
        template_name: &str,
        input_scrap_title: &Option<Title>,
        timezone: &Tz,
    ) -> ScrapsResult<()> {
        let render = MarkdownRender::new(&self.scraps_dir_path, &self.templates_dir_path);

        render.render_from_template(template_name, input_scrap_title, timezone)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
    use rstest::rstest;

    use super::*;
    use std::fs;

    #[rstest]
    fn it_run_has_not_input_template_title(#[from(temp_scrap_project)] project: TempScrapProject) {
        // run args
        let template_name = "it_render_from_template";
        let template_title = &None;
        let timezone = chrono_tz::Asia::Tokyo;

        // template
        let template_bytes =
            "+++\ntitle = \"test_title\"\n+++\n\n{{ \"2019-09-19T15:00:00.000Z\" | date(timezone=timezone) }}".as_bytes();
        project.add_template(&format!("{template_name}.md"), template_bytes);

        // run
        let usecase = GenerateUsecase::new(&project.scraps_dir, &project.templates_dir);
        usecase
            .execute(template_name, template_title, &timezone)
            .unwrap();

        // assert
        let result = fs::read_to_string(project.scrap_path("test_title.md"));
        assert_eq!(result.unwrap(), "\n2019-09-20")
    }

    #[rstest]
    fn it_run_has_input_template_title(#[from(temp_scrap_project)] project: TempScrapProject) {
        // run args
        let template_name = "it_render_from_template";
        let template_title = &Some("override_title".into());
        let timezone = chrono_tz::Asia::Tokyo;

        // template
        let template_bytes =
            "+++\ntitle = \"test_title\"\n+++\n\n{{ \"2019-09-19T15:00:00.000Z\" | date(timezone=timezone) }}".as_bytes();
        project.add_template(&format!("{template_name}.md"), template_bytes);

        // run
        let usecase = GenerateUsecase::new(&project.scraps_dir, &project.templates_dir);
        usecase
            .execute(template_name, template_title, &timezone)
            .unwrap();

        // assert
        let scraps_md_path = format!("{}.md", template_title.as_ref().unwrap());
        let result = fs::read_to_string(project.scrap_path(&scraps_md_path));
        assert_eq!(result.unwrap(), "\n2019-09-20")
    }
}
