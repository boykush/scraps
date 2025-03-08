use std::fs;
use std::path::Path;
use std::{fs::File, path::PathBuf};

use crate::build::model::html::HtmlMetadata;
use crate::build::model::linked_scraps_map::LinkedScrapsMap;
use crate::build::model::scrap_with_commited_ts::ScrapWithCommitedTs;
use chrono_tz::Tz;
use scraps_libs::{
    error::{anyhow::Context, ScrapsError, ScrapResult},
    model::scrap::Scrap,
};
use url::Url;

use crate::build::html::tera::scrap_tera;

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
    ) -> ScrapResult<ScrapRender> {
        let public_scraps_dir_path = &public_dir_path.join("scraps");
        fs::create_dir_all(public_scraps_dir_path).context(ScrapsError::FileWrite)?;

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
    ) -> ScrapResult<()> {
        let (tera, mut context) = scrap_tera::base(
            base_url,
            timezone,
            metadata,
            self.static_dir_path.join("*.html").to_str().unwrap(),
        )?;

        // insert to context for linked list
        let linked_scraps_map = LinkedScrapsMap::new(&self.scraps);
        context.insert("scrap", &ScrapDetailTera::from(scrap_with_commited_ts));

        let linked_scraps = linked_scraps_map.linked_by(&scrap_with_commited_ts.scrap().title);
        context.insert("linked_scraps", &LinkScrapsTera::new(&linked_scraps));

        // render html
        let file_name = &format!("{}.html", &scrap_with_commited_ts.scrap().title.slug);
        let wtr = File::create(self.public_scraps_dir_path.join(file_name))
            .context(ScrapsError::FileWrite)?;
        tera.render_to("__builtins/scrap.html", &context, wtr)
            .context(ScrapsError::PublicRender)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use url::Url;

    use crate::build::model::html::HtmlMetadata;
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
        let scrap1 = &Scrap::new(&base_url, "scrap 1", "# header1");
        let scrap2 = &Scrap::new(&base_url, "scrap 2", "[[scrap1]]");
        let scraps = vec![scrap1.to_owned(), scrap2.to_owned()];

        let scrap1_html_path = public_dir_path.join(format!("scraps/{}.html", scrap1.title.slug));

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
    }
}
