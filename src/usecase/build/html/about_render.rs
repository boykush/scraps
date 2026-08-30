use std::path::{Path, PathBuf};

use crate::error::ScrapsResult;
use crate::service::tera_render::{render_to_file, resolve_template, user_template_glob};
use crate::usecase::build::html::templates;
use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::build::model::html::HtmlMetadata;
use crate::usecase::build::model::site_nav::SiteNav;
use scraps_libs::model::{base_url::BaseUrl, content::Content};
use tera::Tera;

use super::serde::content::ContentTera;

/// A root README.md is the wiki's front matter, not its front page: it
/// renders at /about/ so the home stays a designed listing whatever the
/// README grows into.
pub struct AboutRender {
    tera: Tera,
    output_about_dir_path: PathBuf,
}

impl AboutRender {
    pub fn new(static_dir_path: &Path, output_dir_path: &Path) -> ScrapsResult<AboutRender> {
        let tera = templates::about(&user_template_glob(static_dir_path, "*.html"))?;

        Ok(AboutRender {
            tera,
            output_about_dir_path: output_dir_path.join("about"),
        })
    }

    pub fn run(
        &self,
        base_url: &BaseUrl,
        metadata: &HtmlMetadata,
        readme_content: &Content,
        backlinks_map: &BacklinksMap,
        site_nav: &SiteNav,
    ) -> ScrapsResult<()> {
        let mut context = templates::context(base_url, metadata);
        templates::insert_site_nav(&mut context, "about", site_nav, backlinks_map);
        context.insert("readme_content", &ContentTera::from(readme_content.clone()));

        let template_name = resolve_template(&self.tera, "about.html", "__builtins/about.html");
        let file_path = self.output_about_dir_path.join("index.html");
        render_to_file(&self.tera, template_name, &context, &file_path)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
    use rstest::rstest;
    use scraps_libs::html::{self, EmbedMode};
    use scraps_libs::{
        lang::LangCode, model::base_url::BaseUrl, model::scrap::Scrap, model::tags::Tags,
    };
    use std::collections::HashMap;
    use std::fs;
    use url::Url;

    use super::*;

    #[rstest]
    fn it_run(#[from(temp_scrap_project)] project: TempScrapProject) {
        let base_url = BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let metadata = HtmlMetadata::new(&LangCode::default(), "Scrap", &None, &None);

        let scrap1 = Scrap::new("intro", &None, "# Intro");
        let scraps = [scrap1.clone()];
        let backlinks_map = BacklinksMap::new(&scraps);
        let site_nav = SiteNav::new(scraps.len(), Tags::new(&scraps), true, chrono_tz::UTC, true);

        let scrap_texts: HashMap<_, _> = scraps
            .iter()
            .map(|s| (s.self_key(), s.md_text().to_string()))
            .collect();
        let readme_content = html::to_content(
            "# About this wiki\n\nhttps://example.com/\n",
            &base_url,
            EmbedMode::Expand(&scrap_texts),
        );

        let render = AboutRender::new(&project.static_dir, &project.output_dir).unwrap();
        render
            .run(
                &base_url,
                &metadata,
                &readme_content,
                &backlinks_map,
                &site_nav,
            )
            .unwrap();

        let result = fs::read_to_string(project.output_path("about/index.html")).unwrap();
        assert!(result.contains("About this wiki"));
        assert!(result.contains("link-card"));
        assert!(result.contains(">about<"));
    }
}
