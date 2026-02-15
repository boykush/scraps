use crate::error::ScrapsResult;
use scraps_libs::model::context::Ctx;
use scraps_libs::model::key::ScrapKey;
use scraps_libs::model::scrap::Scrap;
use scraps_libs::model::title::Title;
use std::path::{Path, PathBuf};

/// Result for get scrap operation
#[derive(Debug, Clone, PartialEq)]
pub struct GetScrapResult {
    pub title: Title,
    pub ctx: Option<Ctx>,
    pub md_text: String,
}

pub struct GetScrapUsecase {
    scraps_dir_path: PathBuf,
}

impl GetScrapUsecase {
    pub fn new(scraps_dir_path: &Path) -> GetScrapUsecase {
        GetScrapUsecase {
            scraps_dir_path: scraps_dir_path.to_owned(),
        }
    }

    pub fn execute(&self, title: &Title, ctx: &Option<Ctx>) -> ScrapsResult<GetScrapResult> {
        // Load all scraps from directory
        let scrap_paths = crate::usecase::read_scraps::to_scrap_paths(&self.scraps_dir_path)?;
        let scraps = scrap_paths
            .into_iter()
            .map(|path| crate::usecase::read_scraps::to_scrap_by_path(&self.scraps_dir_path, &path))
            .collect::<ScrapsResult<Vec<Scrap>>>()?;

        // Create scrap key for the target scrap
        let target_key = if let Some(ctx) = ctx {
            ScrapKey::with_ctx(title, ctx)
        } else {
            ScrapKey::from(title.clone())
        };

        // Find the target scrap
        let target_scrap = scraps
            .iter()
            .find(|scrap| scrap.self_key() == target_key)
            .ok_or_else(|| {
                anyhow::anyhow!("Scrap not found: title='{}', ctx='{:?}'", title, ctx)
            })?;

        let scrap_key = target_scrap.self_key();
        let result_title: Title = (&scrap_key).into();
        let result_ctx: Option<Ctx> = (&scrap_key).into();

        Ok(GetScrapResult {
            title: result_title,
            ctx: result_ctx,
            md_text: target_scrap.md_text().to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
    use rstest::rstest;

    #[rstest]
    fn test_get_scrap_success(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_scrap("scrap1.md", b"# Scrap 1\n\nContent of scrap 1.");

        let usecase = GetScrapUsecase::new(&project.scraps_dir);

        let result = usecase
            .execute(&Title::from("scrap1"), &None)
            .expect("Should succeed");

        assert_eq!(result.title.to_string(), "scrap1");
        assert!(result.ctx.is_none());
        assert!(result.md_text.contains("Content of scrap 1"));
    }

    #[rstest]
    fn test_get_scrap_with_context(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_scrap_with_context("Context", "scrap1.md", b"# Scrap 1\n\nContent of scrap 1.");

        let usecase = GetScrapUsecase::new(&project.scraps_dir);

        let result = usecase
            .execute(&Title::from("scrap1"), &Some(Ctx::from("Context")))
            .expect("Should succeed");

        assert_eq!(result.title.to_string(), "scrap1");
        assert_eq!(result.ctx.unwrap().to_string(), "Context");
        assert!(result.md_text.contains("Content of scrap 1"));
    }

    #[rstest]
    fn test_get_scrap_not_found(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_scrap("scrap1.md", b"# Scrap 1\n\nContent.");

        let usecase = GetScrapUsecase::new(&project.scraps_dir);

        let result = usecase.execute(&Title::from("nonexistent"), &None);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Scrap not found"));
    }
}
