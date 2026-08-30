use std::path::{Path, PathBuf};

use crate::error::ScrapsResult;
use crate::service::tera_render::{render_to_file, resolve_template, user_template_glob};
use crate::usecase::build::html::templates;
use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::build::model::html::HtmlMetadata;
use crate::usecase::build::model::scrap_detail::ScrapDetails;
use crate::usecase::build::model::site_nav::SiteNav;
use scraps_libs::model::base_url::BaseUrl;
use tera::Tera;

use super::serde::index_scraps::IndexScrapsTera;

/// The paginated index answers "what changed lately"; this one answers "what
/// is in here", which pagination actively gets in the way of once a wiki runs
/// to hundreds of scraps.
pub struct ScrapsIndexRender {
    tera: Tera,
    output_scraps_dir_path: PathBuf,
}

impl ScrapsIndexRender {
    pub fn new(static_dir_path: &Path, output_dir_path: &Path) -> ScrapsResult<ScrapsIndexRender> {
        let tera = templates::scraps_index(&user_template_glob(static_dir_path, "*.html"))?;

        Ok(ScrapsIndexRender {
            tera,
            output_scraps_dir_path: output_dir_path.join("scraps"),
        })
    }

    pub fn run(
        &self,
        base_url: &BaseUrl,
        metadata: &HtmlMetadata,
        scrap_details: &ScrapDetails,
        backlinks_map: &BacklinksMap,
        site_nav: &SiteNav,
    ) -> ScrapsResult<()> {
        let mut context = templates::context(base_url, metadata);
        templates::insert_site_nav(&mut context, "scraps", site_nav, backlinks_map);
        context.insert(
            "scraps",
            &IndexScrapsTera::new_sorted_by_title(scrap_details, backlinks_map),
        );

        let template_name = resolve_template(
            &self.tera,
            "scraps_index.html",
            "__builtins/scraps_index.html",
        );
        let file_path = self.output_scraps_dir_path.join("index.html");
        render_to_file(&self.tera, template_name, &context, &file_path)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
    use crate::usecase::build::model::scrap_detail::ScrapDetail;
    use rstest::rstest;
    use scraps_libs::{
        lang::LangCode, model::base_url::BaseUrl, model::scrap::Scrap, model::tags::Tags,
    };
    use std::fs;
    use url::Url;

    use super::*;

    #[rstest]
    fn it_run(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_static_file(
            "scraps_index.html",
            b"{% for scrap in scraps %}<a>{{ scrap.title }}:{{ scrap.backlinks_count }}</a>{% endfor %}",
        );

        let base_url = BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let metadata = HtmlMetadata::new(&LangCode::default(), "Scrap", &None, &None);

        // Deliberately out of title order, and mixed case, to pin the sort.
        let scrap1 = Scrap::new("beta", &None, "[[Alpha]]");
        let scrap2 = Scrap::new("Alpha", &None, "");
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
        let site_nav = SiteNav::new(scraps.len(), Tags::new(&scraps), true);

        let render = ScrapsIndexRender::new(&project.static_dir, &project.output_dir).unwrap();
        render
            .run(&base_url, &metadata, &details, &backlinks_map, &site_nav)
            .unwrap();

        let result = fs::read_to_string(project.output_path("scraps/index.html")).unwrap();
        assert_eq!(result, "<a>Alpha:1</a><a>beta:0</a>");
    }
}
