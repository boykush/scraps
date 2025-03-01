use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use chrono_tz::Tz;
use scraps_libs::{
    error::{anyhow::Context, ScrapError, ScrapResult},
    markdown::frontmatter,
    model::title::Title,
};

use crate::template::serde::metadata::TemplateMetadata;

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
    ) -> ScrapResult<()> {
        let (tera, mut context) =
            markdown_tera::init(self.templates_dir_path.join("*.md").to_str().unwrap())?;
        let template_file_name = format!("{}.md", template_name);
        let template = if tera.get_template_names().any(|t| t == template_file_name) {
            Ok(template_file_name.as_str())
        } else {
            Err(ScrapError::NotFoundTemplate)
        }?;

        context.insert("timezone", &timezone);

        let markdown_text = tera
            .render(template, &context)
            .context(ScrapError::PublicRender)?;

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
                    .unwrap_or(Err(ScrapError::RequiredTemplateTitle))
            })?;
        let scrap_file_name = format!("{}.md", scrap_title);
        let ignored_metadata_text = frontmatter::ignore_metadata(&markdown_text);

        let mut wtr = File::create(self.scraps_dir_path.join(&scrap_file_name))
            .context(ScrapError::PublicRender)?;
        wtr.write(ignored_metadata_text.as_bytes())
            .context(ScrapError::PublicRender)?;
        wtr.flush().context(ScrapError::FileWrite)
    }
}
