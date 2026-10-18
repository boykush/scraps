use std::collections::HashMap;
use std::path::Path;

use crate::error::ScrapsResult;
use crate::service::tera_render::{RenderOutput, user_template_glob};
use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::build::model::html::HtmlMetadata;
use crate::usecase::build::model::scrap_detail::{ScrapDetail, ScrapDetails};
use crate::usecase::build::model::scrap_graph::{MAX_NODES, ScrapGraph};
use crate::usecase::build::model::site_nav::SiteNav;
use rayon::prelude::*;
use scraps_libs::model::base_url::BaseUrl;
use scraps_libs::model::key::ScrapKey;
use scraps_libs::model::page_path::PagePath;
use scraps_libs::model::scrap::Scrap;
use tera::Tera;
use tracing::{Level, span};

use crate::usecase::build::html::templates;

use super::serde::link_scraps::LinkScrapsTera;
use super::serde::scrap_detail::ScrapDetailTera;
use super::serde::scrap_graph::ScrapGraphTera;
use super::serde::scrap_type::TypeTera;
use super::serde::source::{ResourceTera, SourceTera};
use super::serde::tag::TagTera;

pub struct ScrapRender {
    tera: Tera,
    output: RenderOutput,
}

impl ScrapRender {
    pub fn new(static_dir_path: &Path, output: &RenderOutput) -> ScrapsResult<ScrapRender> {
        let tera = templates::scrap(&user_template_glob(static_dir_path, "*.html"))?;

        Ok(ScrapRender {
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
        scraps_by_key: &HashMap<ScrapKey, &Scrap>,
        site_nav: &SiteNav,
    ) -> ScrapsResult<()> {
        // The shell is identical on every page and its tag list grows with the
        // wiki, so it is built once and each page clones it.
        let mut shell = templates::context(base_url, metadata);
        templates::insert_site_nav(&mut shell, "", site_nav, backlinks_map);

        scrap_details
            .as_slice()
            .par_iter()
            .try_for_each(|scrap_detail| {
                let _span = span!(Level::INFO, "generate_html_scrap").entered();
                self.render_page(&shell, scrap_detail, backlinks_map, scraps_by_key, site_nav)
            })
    }

    fn render_page(
        &self,
        shell: &tera::Context,
        scrap_detail: &ScrapDetail,
        backlinks_map: &BacklinksMap,
        scraps_by_key: &HashMap<ScrapKey, &Scrap>,
        site_nav: &SiteNav,
    ) -> ScrapsResult<()> {
        let mut context = shell.clone();
        let scrap = scrap_detail.scrap();

        // A scrap that describes itself replaces the site-wide description
        // the shell carries, in every tag `base.html` fills from it.
        if let Some(description) = scrap.description() {
            context.insert("description", description);
        }

        // insert to context for linked list
        context.insert("scrap", &ScrapDetailTera::from(scrap_detail));

        let scrap_tags = scrap
            .tags()
            .iter()
            .map(|tag| TagTera::new(tag, backlinks_map))
            .collect::<Vec<_>>();
        context.insert("scrap_tags", &scrap_tags);

        if let Some(scrap_type) = scrap.scrap_type().and_then(|name| site_nav.types.get(name)) {
            context.insert("scrap_type", &TypeTera::from(scrap_type));
        }

        if let Some(resource) = scrap.resource() {
            context.insert("scrap_resource", &ResourceTera::new(resource));
        }

        let sources = scrap
            .sources()
            .iter()
            .map(SourceTera::from)
            .collect::<Vec<_>>();
        context.insert("sources", &sources);

        let linked_scraps = backlinks_map.get(&scrap.self_key());
        context.insert("linked_scraps", &LinkScrapsTera::new(linked_scraps));

        // Outbound links resolve against the scrap set: a broken link is a
        // lint concern, not a rendering one, so it simply drops out here.
        let outbound_scraps = scrap
            .links()
            .iter()
            .filter_map(|key| scraps_by_key.get(key).copied())
            .collect::<Vec<_>>();
        context.insert("outbound_scraps", &LinkScrapsTera::new(&outbound_scraps));

        // Both lists are already in hand, so the neighborhood costs no extra
        // walk over the wiki.
        if let Some(graph) =
            ScrapGraph::new(scrap.title(), linked_scraps, &outbound_scraps, MAX_NODES)
        {
            context.insert("scrap_graph", &ScrapGraphTera::from(&graph));
        }

        // The path may contain `/`-separated context directories, which
        // the output creates on the way.
        self.output.render(
            &self.tera,
            "__builtins/scrap.html",
            &context,
            PagePath::of_scrap(&scrap.self_key()).as_str(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::usecase::build::model::scrap_types::ScrapTypes;
    use std::fs;
    use std::path::PathBuf;
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
        let link_targets = scraps_libs::html::LinkTargets::new(&scraps);
        let backlinks_map = BacklinksMap::new(&scraps);
        let site_nav = SiteNav::new(
            scraps.len(),
            Tags::new(&scraps),
            ScrapTypes::new(&scraps),
            true,
            chrono_tz::UTC,
            false,
        );
        let scraps_by_key: HashMap<_, _> = scraps
            .iter()
            .map(|scrap| (scrap.self_key(), scrap))
            .collect();

        let scrap1_html_path = output_dir_path.join("scraps/scrap 1.html");
        // The context is a directory, spelled as the key spells it.
        let scrap2_html_path = output_dir_path.join("scraps/Context/scrap 2.html");

        let scrap_details = ScrapDetails::new(vec![
            ScrapDetail::new(scrap1, &commited_ts1, base_url, &link_targets),
            ScrapDetail::new(scrap2, &commited_ts1, base_url, &link_targets),
        ]);
        let render = ScrapRender::new(
            &static_dir_path,
            &RenderOutput::Dir(output_dir_path.clone()),
        )
        .unwrap();

        render
            .run(
                base_url,
                &metadata,
                &scrap_details,
                &backlinks_map,
                &scraps_by_key,
                &site_nav,
            )
            .unwrap();

        let result2 = fs::read_to_string(scrap1_html_path).unwrap();
        assert!(result2.contains(">backlinks &#183; 1"));
        assert!(!result2.contains(">links &#183;"));
        assert!(!result2.contains("corsproxy"));

        let result4 = fs::read_to_string(scrap2_html_path).unwrap();
        assert!(result4.contains(">links &#183; 1"));
        assert!(result4.contains("tags/design.html"));
        assert!(result4.contains("#[["));
    }

    #[test]
    fn it_describes_a_page_by_its_scrap_and_falls_back_to_the_site() {
        let base_url = &BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let metadata = HtmlMetadata::new(
            &LangCode::default(),
            "Scrap",
            &Some("Site wide".to_string()),
            &None,
        );

        let described = &Scrap::new(
            "described",
            &None,
            "---\ndescription: One row per \"completed\" order.\n---\n\nbody",
        );
        let plain = &Scrap::new("plain", &None, "body");
        let scraps = vec![described.to_owned(), plain.to_owned()];
        let link_targets = scraps_libs::html::LinkTargets::new(&scraps);
        let backlinks_map = BacklinksMap::new(&scraps);
        let scraps_by_key: HashMap<_, _> = scraps
            .iter()
            .map(|scrap| (scrap.self_key(), scrap))
            .collect();
        let site_nav = SiteNav::new(
            scraps.len(),
            Tags::new(&scraps),
            ScrapTypes::new(&scraps),
            false,
            chrono_tz::UTC,
            false,
        );
        let scrap_details = ScrapDetails::new(
            [described, plain]
                .into_iter()
                .map(|scrap| ScrapDetail::new(scrap, &None, base_url, &link_targets))
                .collect(),
        );

        let project = crate::test_fixtures::TempScrapProject::new();
        let render = ScrapRender::new(
            &project.static_dir,
            &RenderOutput::Dir(project.output_dir.clone()),
        )
        .unwrap();
        render
            .run(
                base_url,
                &metadata,
                &scrap_details,
                &backlinks_map,
                &scraps_by_key,
                &site_nav,
            )
            .unwrap();

        let read = |stem: &str| {
            fs::read_to_string(project.output_dir.join(format!("scraps/{stem}.html"))).unwrap()
        };

        let own = "One row per &quot;completed&quot; order.";
        let described_html = read("described");
        for tag in [
            format!(r#"<meta name="description" content="{own}" />"#),
            format!(r#"<meta property="og:description" content="{own}" />"#),
            format!(r#"<meta name="twitter:description" content="{own}" />"#),
        ] {
            assert!(described_html.contains(&tag), "missing {tag}");
        }
        assert!(!described_html.contains("Site wide"));

        assert!(read("plain").contains(r#"<meta name="description" content="Site wide" />"#));
    }

    #[test]
    fn it_shows_the_type_only_on_a_typed_scrap() {
        let base_url = &BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let metadata = HtmlMetadata::new(&LangCode::default(), "Scrap", &None, &None);

        let typed = &Scrap::new("typed", &None, "---\ntype: BigQuery Table\n---\nbody");
        let untyped = &Scrap::new("untyped", &None, "#[[BigQuery Table]]");
        let scraps = vec![typed.to_owned(), untyped.to_owned()];
        let link_targets = scraps_libs::html::LinkTargets::new(&scraps);
        let backlinks_map = BacklinksMap::new(&scraps);
        let scraps_by_key: HashMap<_, _> = scraps
            .iter()
            .map(|scrap| (scrap.self_key(), scrap))
            .collect();
        let site_nav = SiteNav::new(
            scraps.len(),
            Tags::new(&scraps),
            ScrapTypes::new(&scraps),
            false,
            chrono_tz::UTC,
            false,
        );
        let scrap_details = ScrapDetails::new(
            [typed, untyped]
                .into_iter()
                .map(|scrap| ScrapDetail::new(scrap, &None, base_url, &link_targets))
                .collect(),
        );

        let project = crate::test_fixtures::TempScrapProject::new();
        ScrapRender::new(
            &project.static_dir,
            &RenderOutput::Dir(project.output_dir.clone()),
        )
        .unwrap()
        .run(
            base_url,
            &metadata,
            &scrap_details,
            &backlinks_map,
            &scraps_by_key,
            &site_nav,
        )
        .unwrap();

        let read = |stem: &str| {
            fs::read_to_string(project.output_dir.join(format!("scraps/{stem}.html"))).unwrap()
        };

        assert!(read("typed").contains(
            "<p class=\"type\">type <a class=\"name\" \
             href=\"http://localhost:1112/types/BigQuery%20Table/\">BigQuery Table</a></p>"
        ));
        // The type is not a tag, so the tag line stays as it was.
        assert!(!read("typed").contains("<p class=\"tags\">"));
        assert!(!read("untyped").contains("<p class=\"type\">"));
    }

    #[test]
    fn it_lists_the_sources_after_the_body_only_when_there_are_some() {
        let base_url = &BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let metadata = HtmlMetadata::new(&LangCode::default(), "Scrap", &None, &None);

        let sourced = &Scrap::new(
            "sourced",
            &None,
            "---\nresource: https://example.com/book\nsources:\n  - resource: https://example.com/book\n    title: A <Book>\n  \
             - resource: https://example.com/blog\n  - resource: plain\n---\n\nbody\n",
        );
        let plain = &Scrap::new("plain", &None, "body");
        let scraps = vec![sourced.to_owned(), plain.to_owned()];
        let link_targets = scraps_libs::html::LinkTargets::new(&scraps);
        let backlinks_map = BacklinksMap::new(&scraps);
        let scraps_by_key: HashMap<_, _> = scraps
            .iter()
            .map(|scrap| (scrap.self_key(), scrap))
            .collect();
        let site_nav = SiteNav::new(
            scraps.len(),
            Tags::new(&scraps),
            ScrapTypes::new(&scraps),
            false,
            chrono_tz::UTC,
            false,
        );
        let scrap_details = ScrapDetails::new(
            [sourced, plain]
                .into_iter()
                .map(|scrap| ScrapDetail::new(scrap, &None, base_url, &link_targets))
                .collect(),
        );

        let project = crate::test_fixtures::TempScrapProject::new();
        ScrapRender::new(
            &project.static_dir,
            &RenderOutput::Dir(project.output_dir.clone()),
        )
        .unwrap()
        .run(
            base_url,
            &metadata,
            &scrap_details,
            &backlinks_map,
            &scraps_by_key,
            &site_nav,
        )
        .unwrap();

        let read = |stem: &str| {
            fs::read_to_string(project.output_dir.join(format!("scraps/{stem}.html"))).unwrap()
        };

        let html = read("sourced");
        assert!(html.contains(">sources &#183; 3"));
        assert!(
            html.contains("<a class=\"link\" href=\"https://example.com/book\">A &lt;Book&gt;</a>")
        );
        assert!(html.contains(
            "<a class=\"link\" href=\"https://example.com/blog\">https://example.com/blog</a>"
        ));
        assert!(html.contains("<span class=\"resource\">plain</span>"));
        assert!(!html.contains("scraps/plain.html"));
        assert!(
            html.find("class=\"content\"").unwrap() < html.find("class=\"source-list\"").unwrap()
        );

        assert!(html.contains(
            "<p class=\"resource\">about <a class=\"link\" href=\"https://example.com/book\">https://example.com/book</a></p>"
        ));

        assert!(!read("plain").contains(">sources &#183;"));
        assert!(!read("plain").contains("<p class=\"resource\">"));
    }

    #[test]
    fn it_draws_the_neighbourhood_only_when_there_is_one() {
        let base_url = &BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let metadata = HtmlMetadata::new(&LangCode::default(), "Scrap", &None, &None);

        let linked = &Scrap::new("linked", &None, "");
        let linking = &Scrap::new("linking", &None, "[[linked]]");
        let alone = &Scrap::new("alone", &None, "no wiki links here");
        let scraps = vec![linked.to_owned(), linking.to_owned(), alone.to_owned()];
        let link_targets = scraps_libs::html::LinkTargets::new(&scraps);
        let backlinks_map = BacklinksMap::new(&scraps);
        let scraps_by_key: HashMap<_, _> = scraps
            .iter()
            .map(|scrap| (scrap.self_key(), scrap))
            .collect();
        let site_nav = SiteNav::new(
            scraps.len(),
            Tags::new(&scraps),
            ScrapTypes::new(&scraps),
            false,
            chrono_tz::UTC,
            false,
        );

        let scrap_details = ScrapDetails::new(
            [linked, linking, alone]
                .into_iter()
                .map(|scrap| ScrapDetail::new(scrap, &None, base_url, &link_targets))
                .collect(),
        );

        let project = crate::test_fixtures::TempScrapProject::new();
        let render = ScrapRender::new(
            &project.static_dir,
            &RenderOutput::Dir(project.output_dir.clone()),
        )
        .unwrap();
        render
            .run(
                base_url,
                &metadata,
                &scrap_details,
                &backlinks_map,
                &scraps_by_key,
                &site_nav,
            )
            .unwrap();

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

    #[test]
    fn it_shows_the_title_and_keeps_the_key_for_paths() {
        let base_url = &BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let metadata = HtmlMetadata::new(&LangCode::default(), "Wiki", &None, &None);

        let orders = &Scrap::new(
            "orders",
            &Some("tables".into()),
            "---\ntitle: Customer Orders\n---\n\nUsed by [[metrics/revenue]].\n",
        );
        let revenue = &Scrap::new(
            "revenue",
            &Some("metrics".into()),
            "See [[tables/orders]] and [[tables/orders|the orders table]].\n",
        );
        let scraps = vec![orders.to_owned(), revenue.to_owned()];
        let link_targets = scraps_libs::html::LinkTargets::new(&scraps);
        let backlinks_map = BacklinksMap::new(&scraps);
        let scraps_by_key: HashMap<_, _> = scraps
            .iter()
            .map(|scrap| (scrap.self_key(), scrap))
            .collect();
        let site_nav = SiteNav::new(
            scraps.len(),
            Tags::new(&scraps),
            ScrapTypes::new(&scraps),
            false,
            chrono_tz::UTC,
            false,
        );
        let scrap_details = ScrapDetails::new(
            [orders, revenue]
                .into_iter()
                .map(|scrap| ScrapDetail::new(scrap, &None, base_url, &link_targets))
                .collect(),
        );

        let project = crate::test_fixtures::TempScrapProject::new();
        let render = ScrapRender::new(
            &project.static_dir,
            &RenderOutput::Dir(project.output_dir.clone()),
        )
        .unwrap();
        render
            .run(
                base_url,
                &metadata,
                &scrap_details,
                &backlinks_map,
                &scraps_by_key,
                &site_nav,
            )
            .unwrap();
        let read = |key: &str| {
            fs::read_to_string(project.output_dir.join(format!("scraps/{key}.html"))).unwrap()
        };

        // The page sits at the key and is named by the title.
        let orders_page = read("tables/orders");
        assert!(orders_page.contains("<h1 class=\"title\">Customer Orders</h1>"));
        assert!(orders_page.contains("<title>Customer Orders - Wiki</title>"));
        assert!(
            orders_page
                .contains("<meta property=\"og:title\" content=\"Customer Orders - Wiki\" />")
        );
        assert!(orders_page.contains("scraps &#47; tables&#47;Customer Orders"));
        assert!(orders_page.contains("<span class=\"title\">revenue</span>"));

        // A link names its target by title unless the author wrote an alias;
        // the link lists and the graph name it the same way.
        let revenue_page = read("metrics/revenue");
        let href = "http://localhost:1112/scraps/tables/orders.html";
        assert!(revenue_page.contains(&format!("<a href=\"{href}\">Customer Orders</a>")));
        assert!(revenue_page.contains(&format!("<a href=\"{href}\">the orders table</a>")));
        assert!(revenue_page.contains("<span class=\"title\">Customer Orders</span>"));
        assert!(revenue_page.contains("<title>tables&#47;Customer Orders</title>"));
        assert!(revenue_page.contains(">Customer Orders</text>"));
        assert!(revenue_page.contains("<h1 class=\"title\">revenue</h1>"));
    }
}
