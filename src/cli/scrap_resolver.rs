use crate::error::ScrapsResult;
use scraps_libs::model::context::Ctx;
use scraps_libs::model::scrap::Scrap;
use scraps_libs::model::title::Title;

pub fn resolve_ctx(
    scraps: &[Scrap],
    title: &Title,
    ctx: Option<&str>,
) -> ScrapsResult<Option<Ctx>> {
    if let Some(c) = ctx {
        return Ok(Some(Ctx::from(c)));
    }

    let candidates: Vec<&Scrap> = scraps.iter().filter(|s| s.title() == title).collect();

    match candidates.as_slice() {
        [] => Ok(None),
        [only] => Ok(Option::<Ctx>::from(&only.self_key())),
        many => {
            let mut listed: Vec<String> = many.iter().map(|s| s.self_key().to_string()).collect();
            listed.sort();
            let joined = listed
                .into_iter()
                .map(|k| format!("  - {k}"))
                .collect::<Vec<_>>()
                .join("\n");
            Err(anyhow::anyhow!(
                "multiple scraps found for \"{title}\". Specify --ctx:\n{joined}"
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_ctx_is_returned() {
        let scraps: Vec<Scrap> = vec![];
        let result = resolve_ctx(&scraps, &Title::from("Auth"), Some("Backend")).unwrap();
        assert_eq!(result, Some(Ctx::from("Backend")));
    }

    #[test]
    fn missing_title_resolves_to_none() {
        let scraps = vec![Scrap::new("rust", &None, "# Rust")];
        let result = resolve_ctx(&scraps, &Title::from("nonexistent"), None).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn unique_title_resolves_to_its_ctx() {
        let scraps = vec![Scrap::new("Auth", &Some("Backend".into()), "# Auth")];
        let result = resolve_ctx(&scraps, &Title::from("Auth"), None).unwrap();
        assert_eq!(result, Some(Ctx::from("Backend")));
    }

    #[test]
    fn ambiguous_title_errors_with_candidates() {
        let scraps = vec![
            Scrap::new("Auth", &Some("Backend".into()), "# Auth"),
            Scrap::new("Auth", &Some("Frontend".into()), "# Auth"),
        ];
        let err = resolve_ctx(&scraps, &Title::from("Auth"), None).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("multiple scraps found"));
        assert!(msg.contains("Backend/Auth"));
        assert!(msg.contains("Frontend/Auth"));
    }
}
