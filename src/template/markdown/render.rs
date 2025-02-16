use std::{
    fs::File,
    path::{Path, PathBuf},
};

use chrono_tz::Tz;
use scraps_libs::{
    error::{anyhow::Context, ScrapError, ScrapResult},
    markdown,
    model::title::Title,
};

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

        let scrap_title = input_scrap_title
            .clone()
            .map(|t| Ok(t.to_string()))
            .unwrap_or({
                let text = tera
                    .render(template, &context)
                    .context(ScrapError::PublicRender)?;
                let metadata_text = markdown::extractor::extract_metadata_text(&text);
                let metadata = scraps_libs::metadata::ScrapMetadata::new(&metadata_text)?;
                metadata
                    .template
                    .map(|t| Ok(t.title))
                    .unwrap_or(Err(ScrapError::RequiredTemplateTitle))
            })?;
        let scrap_file_name = format!("{}.md", scrap_title);

        let wtr = File::create(self.scraps_dir_path.join(&scrap_file_name))
            .context(ScrapError::PublicRender)?;
        tera.render_to(template, &context, wtr)
            .context(ScrapError::PublicRender)
    }
}
