use itertools::Itertools;

use scraps_libs::{
    html::{self, EmbedMode, LinkTargets},
    model::{base_url::BaseUrl, content::Content, scrap::Scrap},
};

#[derive(Clone)]
pub struct ScrapDetail {
    v: Scrap,
    content: Content,
    generated_ts: Option<i64>,
}

impl ScrapDetail {
    pub fn new(
        scrap: &Scrap,
        commited_ts: &Option<i64>,
        base_url: &BaseUrl,
        link_targets: &LinkTargets,
    ) -> ScrapDetail {
        let content = html::to_content(
            scrap.md_text(),
            scrap.ctx(),
            base_url,
            EmbedMode::Expand(link_targets),
        );
        // The frontmatter owns the date; git fills it in when the scrap
        // records none.
        let generated_ts = scrap
            .generated()
            .and_then(|generated| generated.at)
            .map(|at| at.timestamp())
            .or(*commited_ts);
        ScrapDetail {
            v: scrap.to_owned(),
            content,
            generated_ts,
        }
    }

    pub fn scrap(&self) -> &Scrap {
        &self.v
    }

    /// When the content last changed: the frontmatter `generated.at`, or
    /// else the last commit to the file.
    pub fn generated_ts(&self) -> Option<i64> {
        self.generated_ts
    }

    pub fn content(&self) -> &Content {
        &self.content
    }
}

pub struct ScrapDetails(Vec<ScrapDetail>);

impl ScrapDetails {
    pub fn new(scrap_details: Vec<ScrapDetail>) -> ScrapDetails {
        ScrapDetails(scrap_details)
    }

    pub fn as_slice(&self) -> &[ScrapDetail] {
        &self.0
    }

    pub fn to_scraps(&self) -> Vec<Scrap> {
        self.0.iter().map(|sc| sc.scrap().clone()).collect_vec()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use url::Url;

    const COMMITTED: i64 = 1_790_000_000;

    #[rstest]
    #[case::frontmatter_over_git(
        "---\ngenerated: { by: agent/v1, at: 2026-06-20T22:53:05Z }\n---\n\nbody\n",
        Some(COMMITTED),
        Some(1_781_995_985)
    )]
    #[case::frontmatter_without_git(
        "---\ngenerated: { by: agent/v1, at: 2026-06-20T22:53:05Z }\n---\n\nbody\n",
        None,
        Some(1_781_995_985)
    )]
    #[case::git_without_at(
        "---\ngenerated: { by: agent/v1 }\n---\n\nbody\n",
        Some(COMMITTED),
        Some(COMMITTED)
    )]
    #[case::git_without_frontmatter("body\n", Some(COMMITTED), Some(COMMITTED))]
    #[case::neither("body\n", None, None)]
    fn it_dates_a_scrap_by_generated_at_and_else_by_git(
        #[case] text: &str,
        #[case] commited_ts: Option<i64>,
        #[case] expected: Option<i64>,
    ) {
        let base_url = BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let scrap = Scrap::new("post", &None, text);
        let link_targets = LinkTargets::new(std::slice::from_ref(&scrap));
        let detail = ScrapDetail::new(&scrap, &commited_ts, &base_url, &link_targets);
        assert_eq!(detail.generated_ts(), expected);
    }
}
