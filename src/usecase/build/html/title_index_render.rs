use std::path::{Path, PathBuf};

use crate::error::ScrapsResult;
use crate::service::tera_render::{render_to_file, resolve_template, user_template_glob};
use crate::usecase::build::html::templates;
use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::build::model::html::HtmlMetadata;
use crate::usecase::build::model::scrap_detail::ScrapDetails;
use scraps_libs::model::base_url::BaseUrl;
use tera::Tera;

use super::serde::title_index::TitleIndexTera;

/// The title view is an index in the book sense: one page, grouped by the
/// title's initial, so a reader who knows the name can jump straight to it.
pub struct TitleIndexRender {
    tera: Tera,
    output_titles_dir_path: PathBuf,
}

impl TitleIndexRender {
    pub fn new(static_dir_path: &Path, output_dir_path: &Path) -> ScrapsResult<TitleIndexRender> {
        let tera = templates::titles(&user_template_glob(static_dir_path, "*.html"))?;

        Ok(TitleIndexRender {
            tera,
            output_titles_dir_path: output_dir_path.join("titles"),
        })
    }

    pub fn run(
        &self,
        base_url: &BaseUrl,
        metadata: &HtmlMetadata,
        scrap_details: &ScrapDetails,
        backlinks_map: &BacklinksMap,
    ) -> ScrapsResult<()> {
        let mut context = templates::context(base_url, metadata);
        context.insert("groups", &TitleIndexTera::new(scrap_details, backlinks_map));

        let template_name = resolve_template(&self.tera, "titles.html", "__builtins/titles.html");
        let file_path = self.output_titles_dir_path.join("index.html");
        render_to_file(&self.tera, template_name, &context, &file_path)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
    use crate::usecase::build::model::scrap_detail::ScrapDetail;
    use rstest::rstest;
    use scraps_libs::{lang::LangCode, model::base_url::BaseUrl, model::scrap::Scrap};
    use std::fs;
    use url::Url;

    use super::*;

    #[rstest]
    fn it_run(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_static_file(
            "titles.html",
            b"{% for group in groups %}[{{ group.label }}{% for scrap in group.scraps %} {{ scrap.title }}{% endfor %}]{% endfor %}",
        );

        let base_url = BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let metadata = HtmlMetadata::new(&LangCode::default(), "Scrap", &None, &None);

        let scrap1 = Scrap::new("デザイントークン", &None, "");
        let scrap2 = Scrap::new("DTCG", &None, "");
        let scraps = [scrap1.clone(), scrap2.clone()];
        let scrap_texts = scraps
            .iter()
            .map(|s| (s.self_key(), s.md_text().to_string()))
            .collect();

        let details = ScrapDetails::new(&vec![
            ScrapDetail::new(&scrap1, &None, &base_url, &scrap_texts),
            ScrapDetail::new(&scrap2, &None, &base_url, &scrap_texts),
        ]);
        let backlinks_map = BacklinksMap::new(&scraps);

        let render = TitleIndexRender::new(&project.static_dir, &project.output_dir).unwrap();
        render
            .run(&base_url, &metadata, &details, &backlinks_map)
            .unwrap();

        let result = fs::read_to_string(project.output_path("titles/index.html")).unwrap();
        assert_eq!(result, "[た デザイントークン][D DTCG]");
    }

    #[rstest]
    fn builtin_template_renders_groups(#[from(temp_scrap_project)] project: TempScrapProject) {
        let base_url = BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let metadata = HtmlMetadata::new(&LangCode::default(), "Scrap", &None, &None);

        let scrap1 = Scrap::new("デザイントークン", &None, "");
        let scraps = [scrap1.clone()];
        let scrap_texts = scraps
            .iter()
            .map(|s| (s.self_key(), s.md_text().to_string()))
            .collect();

        let details = ScrapDetails::new(&vec![ScrapDetail::new(
            &scrap1,
            &None,
            &base_url,
            &scrap_texts,
        )]);
        let backlinks_map = BacklinksMap::new(&scraps);

        let render = TitleIndexRender::new(&project.static_dir, &project.output_dir).unwrap();
        render
            .run(&base_url, &metadata, &details, &backlinks_map)
            .unwrap();

        let result = fs::read_to_string(project.output_path("titles/index.html")).unwrap();
        assert!(result.contains("class=\"jump\""));
        assert!(result.contains("デザイントークン"));
        assert!(result.contains("scraps/デザイントークン.html") || result.contains("scraps/"));
    }
}
