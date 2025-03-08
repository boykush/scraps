use std::{fs::File, path::PathBuf};

use scraps_libs::error::{anyhow::Context, ScrapError, ScrapResult};
use scraps_libs::model::scrap::Scrap;
use url::Url;

use super::search_index_tera;
use super::serde::search_index_scraps::SearchIndexScrapsTera;

pub struct SearchIndexRender {
    static_dir_path: PathBuf,
    public_dir_path: PathBuf,
}

impl SearchIndexRender {
    pub fn new(static_dir_path: &PathBuf, public_dir_path: &PathBuf) -> SearchIndexRender {
        SearchIndexRender {
            static_dir_path: static_dir_path.to_owned(),
            public_dir_path: public_dir_path.to_owned(),
        }
    }

    pub fn run(&self, base_url: &Url, scraps: &[Scrap]) -> ScrapResult<()> {
        let serialize_scraps = SearchIndexScrapsTera::new(scraps);

        Self::render_search_index_json(self, base_url, &serialize_scraps)
    }

    fn render_search_index_json(
        &self,
        base_url: &Url,
        scraps: &SearchIndexScrapsTera,
    ) -> ScrapResult<()> {
        let (tera, mut context) = search_index_tera::base(
            base_url,
            self.static_dir_path.join("*.json").to_str().unwrap(),
        )?;
        let template_name = if tera.get_template_names().any(|t| t == "search_index.json") {
            "search_index.json"
        } else {
            "__builtins/search_index.json"
        };
        context.insert("scraps", scraps);
        let wtr = File::create(self.public_dir_path.join("search_index.json"))
            .context(ScrapError::PublicRender)?;
        tera.render_to(template_name, &context, wtr)
            .context(ScrapError::PublicRender)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use url::Url;

    use super::*;
    use scraps_libs::model::scrap::Scrap;
    use scraps_libs::tests::{DirResource, FileResource};

    #[test]
    fn it_run() {
        // args
        let base_url = Url::parse("http://localhost:1112/").unwrap();

        let test_resource_path =
            PathBuf::from("tests/resource/build/json/render/it_render_search_index_json");
        let static_dir_path = test_resource_path.join("static");
        let public_dir_path = test_resource_path.join("public");

        // static
        let template_json_path = static_dir_path.join("search_index.json");
        let resource_template_json = FileResource::new(&template_json_path);
        let resource_template_json_byte =
        "[{% for scrap in scraps %}{ \"title\": \"{{ scrap.title }}\", \"url\": \"{{ base_url}}scraps/{{ scrap.slug }}.html\" }{% if not loop.last %},{% endif %}{% endfor %}]"
        .as_bytes();

        // public
        let resource_public_dir = DirResource::new(&public_dir_path);

        // scraps
        let sc1 = Scrap::new(&base_url, "scrap1", "# header1");
        let sc2 = Scrap::new(&base_url, "scrap2", "## header2");
        let scraps = vec![sc1, sc2];

        let search_index_json_path = public_dir_path.join("search_index.json");

        resource_template_json.run(resource_template_json_byte, || {
            resource_public_dir.run(|| {
                let render = SearchIndexRender::new(&static_dir_path, &public_dir_path);
                let result1 = render.run(&base_url, &scraps);

                assert!(result1.is_ok());

                let result2 = fs::read_to_string(search_index_json_path).unwrap();
                assert_eq!(
                    result2,
                    "[{ \"title\": \"scrap1\", \"url\": \"http://localhost:1112/scraps/scrap1.html\" },{ \"title\": \"scrap2\", \"url\": \"http://localhost:1112/scraps/scrap2.html\" }]");
            })
        })
    }
}
