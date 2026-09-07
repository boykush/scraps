use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::error::ScrapsResult;
use crate::service::tera_render::{render_to_file, user_template_glob};
use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::build::model::html::HtmlMetadata;
use crate::usecase::build::model::scrap_detail::ScrapDetail;
use crate::usecase::build::model::scrap_graph::{ScrapGraph, MAX_NODES};
use crate::usecase::build::model::site_nav::SiteNav;
use scraps_libs::model::base_url::BaseUrl;
use scraps_libs::model::file::ScrapFileStem;
use scraps_libs::model::key::ScrapKey;
use scraps_libs::model::scrap::Scrap;
use tera::Tera;

use crate::usecase::build::html::templates;

use super::serde::link_scraps::LinkScrapsTera;
use super::serde::scrap_detail::ScrapDetailTera;
use super::serde::scrap_graph::ScrapGraphTera;
use super::serde::tag::TagTera;

pub struct ScrapRender {
    tera: Tera,
    output_scraps_dir_path: PathBuf,
}

impl ScrapRender {
    pub fn new(static_dir_path: &Path, output_dir_path: &Path) -> ScrapsResult<ScrapRender> {
        let tera = templates::scrap(&user_template_glob(static_dir_path, "*.html"))?;

        Ok(ScrapRender {
            tera,
            output_scraps_dir_path: output_dir_path.join("scraps"),
        })
    }

    pub fn run(
        &self,
        base_url: &BaseUrl,
        metadata: &HtmlMetadata,
        scrap_detail: &ScrapDetail,
        backlinks_map: &BacklinksMap,
        scraps_by_key: &HashMap<ScrapKey, Scrap>,
        site_nav: &SiteNav,
    ) -> ScrapsResult<()> {
        let mut context = templates::context(base_url, metadata);
        templates::insert_site_nav(&mut context, "", site_nav, backlinks_map);
        let scrap = &scrap_detail.scrap();

        // insert to context for linked list
        context.insert("scrap", &ScrapDetailTera::from(scrap_detail.clone()));

        let scrap_tags = scrap
            .tags()
            .iter()
            .map(|tag| TagTera::new(tag, backlinks_map))
            .collect::<Vec<_>>();
        context.insert("scrap_tags", &scrap_tags);

        let linked_scraps = backlinks_map.get(&scrap.self_key());
        context.insert("linked_scraps", &LinkScrapsTera::new(&linked_scraps));

        // Outbound links resolve against the scrap set: a broken link is a
        // lint concern, not a rendering one, so it simply drops out here.
        let mut seen = HashSet::new();
        let outbound_scraps = scrap
            .links()
            .iter()
            .filter(|key| seen.insert((*key).clone()))
            .filter_map(|key| scraps_by_key.get(key).cloned())
            .collect::<Vec<_>>();
        context.insert("outbound_scraps", &LinkScrapsTera::new(&outbound_scraps));

        // Both lists are already in hand, so the neighborhood costs no extra
        // walk over the wiki.
        if let Some(graph) = ScrapGraph::new(
            &scrap.title().to_string(),
            &linked_scraps,
            &outbound_scraps,
            MAX_NODES,
        ) {
            context.insert("scrap_graph", &ScrapGraphTera::from(&graph));
        }

        // The stem may contain `/`-separated context directories, which
        // `render_to_file` creates on the way.
        let file_path = self
            .output_scraps_dir_path
            .join(format!("{}.html", ScrapFileStem::from(scrap.self_key())));
        render_to_file(&self.tera, "__builtins/scrap.html", &context, &file_path)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use url::Url;

    use crate::usecase::build::model::backlinks_map::BacklinksMap;
    use crate::usecase::build::model::html::HtmlMetadata;
    use scraps_libs::lang::LangCode;
    use scraps_libs::model::base_url::BaseUrl;
    use scraps_libs::model::scrap::Scrap;
    use scraps_libs::model::tags::Tags;

    use super::*;

    #[test]
    fn it_run() {
        // args
        let base_url = &BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let metadata = HtmlMetadata::new(
            &LangCode::default(),
            "Scrap",
            &Some("Scrap Wiki".to_string()),
            &Some(Url::parse("https://github.io/image.png").unwrap()),
        );

        let test_resource_path =
            PathBuf::from("tests/resource/build/html/render/it_render_scrap_htmls");
        let static_dir_path = test_resource_path.join("static");
        let output_dir_path = test_resource_path.join("_site");

        // scraps
        let commited_ts1 = None;
        let scrap1 = &Scrap::new("scrap 1", &None, "# header1");
        let scrap2 = &Scrap::new(
            "scrap 2",
            &Some("Context".into()),
            "[[scrap 1]] #[[design]]",
        );
        let scraps = vec![scrap1.to_owned(), scrap2.to_owned()];
        let scrap_texts = scraps
            .iter()
            .map(|scrap| (scrap.self_key(), scrap.md_text().to_string()))
            .collect();
        let backlinks_map = BacklinksMap::new(&scraps);
        let site_nav = SiteNav::new(
            scraps.len(),
            Tags::new(&scraps),
            true,
            chrono_tz::UTC,
            false,
        );
        let scraps_by_key: HashMap<_, _> = scraps
            .iter()
            .map(|scrap| (scrap.self_key(), scrap.clone()))
            .collect();

        let scrap1_html_path = output_dir_path.join("scraps/scrap-1.html");
        // v1: nested ctx is a directory (`context/scrap-2.html`), not a
        // dot-suffix on the file stem.
        let scrap2_html_path = output_dir_path.join("scraps/context/scrap-2.html");

        let render = ScrapRender::new(&static_dir_path, &output_dir_path).unwrap();

        render
            .run(
                base_url,
                &metadata,
                &ScrapDetail::new(scrap1, &commited_ts1, base_url, &scrap_texts),
                &backlinks_map,
                &scraps_by_key,
                &site_nav,
            )
            .unwrap();

        let result2 = fs::read_to_string(scrap1_html_path).unwrap();
        assert!(result2.contains(">backlinks &#183; 1"));
        assert!(!result2.contains(">links &#183;"));
        assert!(!result2.contains("corsproxy"));

        render
            .run(
                base_url,
                &metadata,
                &ScrapDetail::new(scrap2, &commited_ts1, base_url, &scrap_texts),
                &backlinks_map,
                &scraps_by_key,
                &site_nav,
            )
            .unwrap();

        let result4 = fs::read_to_string(scrap2_html_path).unwrap();
        assert!(result4.contains(">links &#183; 1"));
        assert!(result4.contains("tags/design.html"));
        assert!(result4.contains("#[["));
    }

    #[test]
    fn it_draws_the_neighbourhood_only_when_there_is_one() {
        let base_url = &BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let metadata = HtmlMetadata::new(&LangCode::default(), "Scrap", &None, &None);

        let linked = &Scrap::new("linked", &None, "");
        let linking = &Scrap::new("linking", &None, "[[linked]]");
        let alone = &Scrap::new("alone", &None, "no wiki links here");
        let scraps = vec![linked.to_owned(), linking.to_owned(), alone.to_owned()];
        let scrap_texts = scraps
            .iter()
            .map(|scrap| (scrap.self_key(), scrap.md_text().to_string()))
            .collect();
        let backlinks_map = BacklinksMap::new(&scraps);
        let scraps_by_key: HashMap<_, _> = scraps
            .iter()
            .map(|scrap| (scrap.self_key(), scrap.clone()))
            .collect();
        let site_nav = SiteNav::new(
            scraps.len(),
            Tags::new(&scraps),
            false,
            chrono_tz::UTC,
            false,
        );

        let project = crate::test_fixtures::TempScrapProject::new();
        let render = ScrapRender::new(&project.static_dir, &project.output_dir).unwrap();
        let render_one = |scrap: &Scrap| {
            render
                .run(
                    base_url,
                    &metadata,
                    &ScrapDetail::new(scrap, &None, base_url, &scrap_texts),
                    &backlinks_map,
                    &scraps_by_key,
                    &site_nav,
                )
                .unwrap();
        };

        render_one(linked);
        render_one(linking);
        render_one(alone);

        let read = |stem: &str| {
            fs::read_to_string(project.output_dir.join(format!("scraps/{stem}.html"))).unwrap()
        };

        // The backlink points at the scrap being read, so the arrow lands on
        // the centre and no arrow is drawn at the neighbour.
        let backlinked = read("linked");
        assert!(backlinked.contains("<svg class=\"scrap-graph\""));
        assert!(backlinked.contains("marker-start=\"url(#scrap-graph-arrow)\""));
        assert!(!backlinked.contains("marker-end=\"url(#scrap-graph-arrow)\""));

        let outbound = read("linking");
        assert!(outbound.contains("marker-end=\"url(#scrap-graph-arrow)\""));
        assert!(!outbound.contains("marker-start=\"url(#scrap-graph-arrow)\""));

        assert!(!read("alone").contains("scrap-graph"));
    }
}
