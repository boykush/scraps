use std::fs;
use std::path::Path;
use std::{fs::File, path::PathBuf};

use crate::error::BuildError;
use crate::error::{anyhow::Context, ScrapsResult};
use crate::usecase::build::model::html::HtmlMetadata;
use crate::usecase::build::model::linked_scraps_map::LinkedScrapsMap;
use crate::usecase::build::model::scrap_with_commited_ts::ScrapWithCommitedTs;
use chrono_tz::Tz;
use scraps_libs::model::file::ScrapFileStem;
use scraps_libs::model::scrap::Scrap;
use url::Url;

use crate::usecase::build::html::tera::scrap_tera;

use super::serde::link_scraps::LinkScrapsTera;
use super::serde::scrap_detail::ScrapDetailTera;

pub struct ScrapRender {
    static_dir_path: PathBuf,
    public_scraps_dir_path: PathBuf,
    scraps: Vec<Scrap>,
}

impl ScrapRender {
    pub fn new(
        static_dir_path: &Path,
        public_dir_path: &Path,
        scraps: &Vec<Scrap>,
    ) -> ScrapsResult<ScrapRender> {
        let public_scraps_dir_path = &public_dir_path.join("scraps");
        fs::create_dir_all(public_scraps_dir_path).context(BuildError::CreateDir)?;

        Ok(ScrapRender {
            static_dir_path: static_dir_path.to_owned(),
            public_scraps_dir_path: public_scraps_dir_path.to_owned(),
            scraps: scraps.to_owned(),
        })
    }

    pub fn run(
        &self,
        base_url: &Url,
        timezone: Tz,
        metadata: &HtmlMetadata,
        scrap_with_commited_ts: &ScrapWithCommitedTs,
    ) -> ScrapsResult<()> {
        let (tera, mut context) = scrap_tera::base(
            base_url,
            timezone,
            metadata,
            self.static_dir_path.join("*.html").to_str().unwrap(),
        )?;
        let scrap = &scrap_with_commited_ts.scrap();

        // insert to context for linked list
        let linked_scraps_map = LinkedScrapsMap::new(&self.scraps);
        context.insert("scrap", &ScrapDetailTera::from(scrap_with_commited_ts));

        let linked_scraps = linked_scraps_map.linked_by(&scrap.self_link());
        context.insert("linked_scraps", &LinkScrapsTera::new(&linked_scraps));

        let file_path = &self
            .public_scraps_dir_path
            .join(format!("{}.html", ScrapFileStem::from(scrap.self_link())));
        let wtr = File::create(file_path).context(BuildError::WriteFailure(file_path.clone()))?;
        tera.render_to("__builtins/scrap.html", &context, wtr)
            .context(BuildError::WriteFailure(file_path.clone()))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use url::Url;

    use crate::usecase::build::model::html::HtmlMetadata;
    use scraps_libs::lang::LangCode;

    use super::*;

    #[test]
    fn it_run() {
        // args
        let base_url = Url::parse("http://localhost:1112/").unwrap();
        let timezone = chrono_tz::UTC;
        let metadata = HtmlMetadata::new(
            &LangCode::default(),
            "Scrap",
            &Some("Scrap Wiki".to_string()),
            &Some(Url::parse("https://github.io/image.png").unwrap()),
        );

        let test_resource_path =
            PathBuf::from("tests/resource/build/html/render/it_render_scrap_htmls");
        let static_dir_path = test_resource_path.join("static");
        let public_dir_path = test_resource_path.join("public");

        // scraps
        let commited_ts1 = None;
        let scrap1 = &Scrap::new(&base_url, "scrap 1", &None, "# header1");
        let scrap2 = &Scrap::new(&base_url, "scrap 2", &Some("Context"), "[[scrap1]]");
        let scraps = vec![scrap1.to_owned(), scrap2.to_owned()];

        let scrap1_html_path = public_dir_path.join("scraps/scrap-1.html");
        let scrap2_html_path = public_dir_path.join("scraps/scrap-2.context.html");

        let render = ScrapRender::new(&static_dir_path, &public_dir_path, &scraps).unwrap();

        let result1 = render.run(
            &base_url,
            timezone,
            &metadata,
            &ScrapWithCommitedTs::new(scrap1, &commited_ts1),
        );
        assert!(result1.is_ok());

        let result2 = fs::read_to_string(scrap1_html_path);
        assert!(result2.is_ok());

        let result3 = render.run(
            &base_url,
            timezone,
            &metadata,
            &ScrapWithCommitedTs::new(scrap2, &commited_ts1),
        );
        assert!(result3.is_ok());

        let result4 = fs::read_to_string(scrap2_html_path);
        assert!(result4.is_ok());
    }
}
