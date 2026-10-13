use crate::error::ScrapsResult;
use scraps_libs::model::context::Ctx;
use scraps_libs::model::key::ScrapKey;
use scraps_libs::model::scrap::Scrap;

/// How an output names a scrap: the key a caller passes back to reach it, and
/// the title a reader sees. The context is read off the key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrapRef {
    pub key: ScrapKey,
    pub title: String,
}

impl ScrapRef {
    pub fn ctx(&self) -> &Option<Ctx> {
        self.key.ctx()
    }

    /// Context first, then stem: the order listings have always come in.
    pub fn sort_key(&self) -> (String, String) {
        (
            self.ctx()
                .as_ref()
                .map(|c| c.to_string())
                .unwrap_or_default(),
            self.key.stem().to_string(),
        )
    }
}

impl From<&Scrap> for ScrapRef {
    fn from(scrap: &Scrap) -> Self {
        ScrapRef {
            key: scrap.self_key(),
            title: scrap.title().to_string(),
        }
    }
}

/// The scrap a caller named by key, or the error every lookup reports when
/// no file sits at that key.
pub fn find_scrap<'a>(scraps: &'a [Scrap], key: &ScrapKey) -> ScrapsResult<&'a Scrap> {
    scraps
        .iter()
        .find(|scrap| &scrap.self_key() == key)
        .ok_or_else(|| anyhow::anyhow!("Scrap not found: key='{key}'"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_names_a_scrap_by_key_and_title() {
        let scrap = Scrap::new(
            "orders",
            &Some("tables".into()),
            "---\ntitle: Customer Orders\n---\n",
        );
        let scrap_ref = ScrapRef::from(&scrap);
        assert_eq!(scrap_ref.key.to_string(), "tables/orders");
        assert_eq!(scrap_ref.title, "Customer Orders");
        assert_eq!(scrap_ref.ctx(), &Some("tables".into()));
    }

    #[test]
    fn it_titles_a_scrap_without_a_frontmatter_title_by_its_stem() {
        let scrap_ref = ScrapRef::from(&Scrap::new("orders", &Some("tables".into()), ""));
        assert_eq!(scrap_ref.title, "orders");
    }

    #[test]
    fn it_finds_by_key_and_never_by_title() {
        let scraps = vec![Scrap::new(
            "orders",
            &Some("tables".into()),
            "---\ntitle: Customer Orders\n---\n",
        )];
        assert!(find_scrap(&scraps, &ScrapKey::from_path_str("tables/orders")).is_ok());
        let by_title = find_scrap(&scraps, &ScrapKey::from_path_str("Customer Orders"));
        assert!(
            by_title
                .unwrap_err()
                .to_string()
                .contains("Scrap not found")
        );
    }
}
