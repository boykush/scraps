use std::{fs::File, path::PathBuf};

use crate::error::BuildError;
use crate::error::{anyhow::Context, ScrapsResult};
use scraps_libs::model::base_url::BaseUrl;
use scraps_libs::model::scrap::Scrap;

use super::search_index_tera;
use super::serde::search_index_scraps::SearchIndexScrapsTera;

pub struct SearchIndexRender {
    static_dir_path: PathBuf,
    public_dir_path: PathBuf,
}

impl SearchIndexRender {
    pub fn new(
        static_dir_path: &PathBuf,
        public_dir_path: &PathBuf,
    ) -> ScrapsResult<SearchIndexRender> {
        std::fs::create_dir_all(public_dir_path).context(BuildError::CreateDir)?;

        Ok(SearchIndexRender {
            static_dir_path: static_dir_path.to_owned(),
            public_dir_path: public_dir_path.to_owned(),
        })
    }

    pub fn run(&self, base_url: &BaseUrl, scraps: &[Scrap]) -> ScrapsResult<()> {
        let serialize_scraps = SearchIndexScrapsTera::new(scraps);

        Self::render_search_index_json(self, base_url, &serialize_scraps)
    }

    fn render_search_index_json(
        &self,
        base_url: &BaseUrl,
        scraps: &SearchIndexScrapsTera,
    ) -> ScrapsResult<()> {
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
        let file_path = &self.public_dir_path.join("search_index.json");
        let wtr = File::create(file_path).context(BuildError::WriteFailure(file_path.clone()))?;
        tera.render_to(template_name, &context, wtr)
            .context(BuildError::WriteFailure(file_path.clone()))
    }
}

#[cfg(test)]
mod tests {
    use crate::test_fixtures::TempScrapProject;
    use std::fs;
    use url::Url;

    use super::*;
    use scraps_libs::model::scrap::Scrap;

    #[test]
    fn it_run() {
        let project = TempScrapProject::new();

        // Add static search_index.json template
        project.add_static_file(
            "search_index.json",
            b"[{% for scrap in scraps %}{ \"title\": \"{{ scrap.link_title }}\", \"url\": \"{{ base_url}}scraps/{{ scrap.file_stem }}.html\" }{% if not loop.last %},{% endif %}{% endfor %}]"
        );

        let base_url = BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();

        // Create scraps
        let sc1 = Scrap::new("scrap1", &None, "# header1");
        let sc2 = Scrap::new("scrap2", &Some("Context"), "## header2");
        let scraps = vec![sc1, sc2];

        let render = SearchIndexRender::new(&project.static_dir, &project.public_dir).unwrap();
        render.run(&base_url, &scraps).unwrap();

        let result = fs::read_to_string(project.public_path("search_index.json")).unwrap();
        assert_eq!(
            result,
            "[{ \"title\": \"scrap1\", \"url\": \"http://localhost:1112/scraps/scrap1.html\" },{ \"title\": \"Context/scrap2\", \"url\": \"http://localhost:1112/scraps/scrap2.context.html\" }]");
    }
}
