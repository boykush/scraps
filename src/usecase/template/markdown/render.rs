use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use crate::error::{anyhow::Context, ScrapsResult, TemplateError};
use chrono_tz::Tz;
use scraps_libs::{markdown::frontmatter, model::title::Title};

use crate::usecase::template::serde::metadata::TemplateMetadata;

use super::markdown_tera;

pub struct MarkdownRender {
    scraps_dir_path: PathBuf,
    templates_dir_path: PathBuf,
}

impl MarkdownRender {
    pub fn new(scraps_dir_path: &Path, templates_dir_path: &Path) -> MarkdownRender {
        MarkdownRender {
            scraps_dir_path: scraps_dir_path.to_path_buf(),
            templates_dir_path: templates_dir_path.to_path_buf(),
        }
    }

    pub fn render_from_template(
        &self,
        template_name: &str,
        input_scrap_title: &Option<Title>,
        timezone: &Tz,
    ) -> ScrapsResult<()> {
        let (tera, mut context) =
            markdown_tera::base(self.templates_dir_path.join("*.md").to_str().unwrap())?;
        let template_file_name = format!("{}.md", template_name);
        let template = if tera.get_template_names().any(|t| t == template_file_name) {
            Ok(template_file_name.as_str())
        } else {
            Err(TemplateError::NotFound(template_name.to_string()))
        }?;

        context.insert("timezone", &timezone);

        let markdown_text = tera
            .render(template, &context)
            .context(TemplateError::RenderFailure)?;

        let scrap_title = input_scrap_title
            .clone()
            .map(|t| Ok(t.to_string()))
            .unwrap_or({
                let metadata_text = frontmatter::get_metadata_text(&markdown_text);
                let metadata_result = metadata_text
                    .map(|t| TemplateMetadata::new(&t))
                    .transpose()?;
                metadata_result
                    .map(|t| Ok(t.title))
                    .unwrap_or(Err(TemplateError::RequiredTitle))
            })?;
        let scrap_file_name = format!("{}.md", scrap_title);
        let ignored_metadata_text = frontmatter::ignore_metadata(&markdown_text);

        let mut wtr = File::create(self.scraps_dir_path.join(&scrap_file_name))
            .context(TemplateError::WriteFailure)?;
        wtr.write(ignored_metadata_text.as_bytes())
            .context(TemplateError::WriteFailure)?;
        wtr.flush().context(TemplateError::WriteFailure)
    }
}
