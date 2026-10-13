use std::path::Path;

use crate::error::ScrapsResult;
use crate::service::tera_render::{RenderOutput, resolve_template, user_template_glob};
use crate::usecase::build::html::templates;
use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::build::model::html::HtmlMetadata;
use crate::usecase::build::model::scrap_detail::ScrapDetails;
use crate::usecase::build::model::site_nav::SiteNav;
use scraps_libs::model::base_url::BaseUrl;
use tera::Tera;

use super::serde::title_index::TitleIndexTera;

/// The paginated index answers "what changed lately"; this one answers "what
/// is in here": every scrap on a single page, grouped by the title's initial
/// with a jump bar, so a reader who knows the name goes straight to it.
// Not `scraps/`: that directory holds one page per key, and a listing inside
// it would sit where a scrap named `index` is written.
const VIEW_PATH: &str = "all-scraps/";

pub struct ScrapsIndexRender {
    tera: Tera,
    output: RenderOutput,
}

impl ScrapsIndexRender {
    pub fn new(static_dir_path: &Path, output: &RenderOutput) -> ScrapsResult<ScrapsIndexRender> {
        let tera = templates::scraps_index(&user_template_glob(static_dir_path, "*.html"))?;

        Ok(ScrapsIndexRender {
            tera,
            output: output.clone(),
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
        let mut shell = templates::context(base_url, metadata);
        templates::insert_site_nav(&mut shell, "scraps", site_nav, backlinks_map);
        let template_name = resolve_template(
            &self.tera,
            "scraps_index.html",
            "__builtins/scraps_index.html",
        );

        let mut context = shell.clone();
        templates::insert_type_filter(&mut context, VIEW_PATH, site_nav, None);
        context.insert("groups", &TitleIndexTera::new(scrap_details, backlinks_map));
        self.output
            .join(VIEW_PATH)
            .render(&self.tera, template_name, &context, "index.html")?;

        // The same index again for each type, under the type's directory.
        site_nav.types.iter().try_for_each(|scrap_type| {
            let mut context = shell.clone();
            templates::insert_type_filter(&mut context, VIEW_PATH, site_nav, Some(scrap_type));
            let groups = TitleIndexTera::of_type(scrap_details, backlinks_map, &scrap_type.name);
            context.insert("groups", &groups);
            self.output
                .join(scrap_type.page_path().as_str())
                .join(VIEW_PATH)
                .render(&self.tera, template_name, &context, "index.html")
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::test_fixtures::{TempScrapProject, temp_scrap_project};
    use crate::usecase::build::model::scrap_detail::ScrapDetail;
    use crate::usecase::build::model::scrap_types::ScrapTypes;
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
            b"{% for group in groups %}[{{ group.label }}{% for scrap in group.scraps %} {{ scrap.title }}:{{ scrap.backlinks_count }}{% endfor %}]{% endfor %}",
        );

        let base_url = BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let metadata = HtmlMetadata::new(&LangCode::default(), "Scrap", &None, &None);

        // Mixed initials and mixed case, to pin grouping and in-group order.
        let scrap1 = Scrap::new("beta", &None, "[[Alpha]]");
        let scrap2 = Scrap::new("Alpha", &None, "");
        let scrap3 = Scrap::new("デザイントークン", &None, "");
        let scraps = [scrap1.clone(), scrap2.clone(), scrap3.clone()];
        let link_targets = scraps_libs::html::LinkTargets::new(&scraps);

        let details = ScrapDetails::new(vec![
            ScrapDetail::new(&scrap1, &None, &base_url, &link_targets),
            ScrapDetail::new(&scrap2, &None, &base_url, &link_targets),
            ScrapDetail::new(&scrap3, &None, &base_url, &link_targets),
        ]);
        let backlinks_map = BacklinksMap::new(&scraps);
        let site_nav = SiteNav::new(
            scraps.len(),
            Tags::new(&scraps),
            ScrapTypes::new(&scraps),
            true,
            chrono_tz::UTC,
            false,
        );

        let render = ScrapsIndexRender::new(
            &project.static_dir,
            &RenderOutput::Dir(project.output_dir.clone()),
        )
        .unwrap();
        render
            .run(&base_url, &metadata, &details, &backlinks_map, &site_nav)
            .unwrap();

        let result = fs::read_to_string(project.output_path("all-scraps/index.html")).unwrap();
        assert_eq!(result, "[た デザイントークン:0][A Alpha:1][B beta:0]");
    }

    #[rstest]
    fn builtin_template_renders_jump_bar(#[from(temp_scrap_project)] project: TempScrapProject) {
        let base_url = BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let metadata = HtmlMetadata::new(&LangCode::default(), "Scrap", &None, &None);

        let scrap1 = Scrap::new("デザイントークン", &None, "");
        let scraps = [scrap1.clone()];
        let link_targets = scraps_libs::html::LinkTargets::new(&scraps);

        let details = ScrapDetails::new(vec![ScrapDetail::new(
            &scrap1,
            &None,
            &base_url,
            &link_targets,
        )]);
        let backlinks_map = BacklinksMap::new(&scraps);
        let site_nav = SiteNav::new(
            scraps.len(),
            Tags::new(&scraps),
            ScrapTypes::new(&scraps),
            true,
            chrono_tz::UTC,
            false,
        );

        let render = ScrapsIndexRender::new(
            &project.static_dir,
            &RenderOutput::Dir(project.output_dir.clone()),
        )
        .unwrap();
        render
            .run(&base_url, &metadata, &details, &backlinks_map, &site_nav)
            .unwrap();

        let result = fs::read_to_string(project.output_path("all-scraps/index.html")).unwrap();
        assert!(result.contains("class=\"jump\""));
        assert!(result.contains("デザイントークン"));
    }

    #[rstest]
    fn it_narrows_the_index_to_a_type(#[from(temp_scrap_project)] project: TempScrapProject) {
        let base_url = BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let metadata = HtmlMetadata::new(&LangCode::default(), "Scrap", &None, &None);
        let scraps = [
            Scrap::new("alpha", &None, "---\ntype: Blog\n---\n"),
            Scrap::new("beta", &None, "---\ntype: Field Note\n---\n"),
            Scrap::new("gamma", &None, ""),
        ];
        let link_targets = scraps_libs::html::LinkTargets::new(&scraps);
        let scrap_details = ScrapDetails::new(
            scraps
                .iter()
                .map(|scrap| ScrapDetail::new(scrap, &None, &base_url, &link_targets))
                .collect(),
        );
        let site_nav = SiteNav::new(
            scraps.len(),
            Tags::new(&scraps),
            ScrapTypes::new(&scraps),
            false,
            chrono_tz::UTC,
            false,
        );

        ScrapsIndexRender::new(
            &project.static_dir,
            &RenderOutput::Dir(project.output_dir.clone()),
        )
        .unwrap()
        .run(
            &base_url,
            &metadata,
            &scrap_details,
            &BacklinksMap::new(&scraps),
            &site_nav,
        )
        .unwrap();

        let read = |path: &str| fs::read_to_string(project.output_path(path)).unwrap();

        let all = read("all-scraps/index.html");
        assert!(all.contains("href=\"http://localhost:1112/types/Blog/all-scraps/\">Blog<"));
        assert!(
            all.contains(
                "href=\"http://localhost:1112/types/Field%20Note/all-scraps/\">Field Note<"
            )
        );
        assert!(all.contains("scraps/gamma.html"));

        // The listing stays out of `scraps/`, where a key could name it.
        assert!(!project.output_path("scraps").exists());

        let upper = read("types/Blog/all-scraps/index.html");
        assert!(upper.contains("scraps/alpha.html"));
        assert!(!upper.contains("scraps/beta.html"));
        assert!(upper.contains("<span class=\"stats\">1 of 3 scraps</span>"));

        let other = read("types/Field Note/all-scraps/index.html");
        assert!(other.contains("scraps/beta.html"));
        assert!(!other.contains("scraps/alpha.html"));
    }
}
