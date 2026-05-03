use std::path::PathBuf;

use rayon::prelude::*;
use scraps_libs::git::GitCommand;
use scraps_libs::model::{scrap::Scrap, tags::Tags};

use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::lint::rule::{scrap_relative_path, LintRule, LintRuleName, LintWarning};

const SECONDS_PER_DAY: i64 = 86_400;

/// Flag scraps whose latest git commit is older than `threshold_days`.
///
/// Maps to Notion's "Verified Pages" concept: knowledge bases accumulate
/// content that was once accurate but rots when left untouched. The default
/// 180-day threshold matches Notion's default re-verification cycle.
///
/// Opt-in by rule name (`--rule stale-by-git` or `--all`) so that the default
/// `scraps lint` invocation stays git-free and fast.
pub struct StaleByGitRule<GC: GitCommand> {
    pub git_command: GC,
    pub scraps_dir: PathBuf,
    pub threshold_days: u64,
    pub now_ts: i64,
}

impl<GC: GitCommand + Send + Sync> LintRule for StaleByGitRule<GC> {
    fn name(&self) -> LintRuleName {
        LintRuleName::StaleByGit
    }

    fn check(
        &self,
        scraps: &[Scrap],
        _backlinks_map: &BacklinksMap,
        _tags: &Tags,
    ) -> Vec<LintWarning> {
        let is_repo = self.git_command.is_git_repository(&self.scraps_dir);
        match is_repo {
            Ok(true) => {}
            Ok(false) => {
                eprintln!("info: stale-by-git: git unavailable, skipping stale check");
                return Vec::new();
            }
            Err(e) => {
                eprintln!(
                    "info: stale-by-git: git unavailable ({}), skipping stale check",
                    e
                );
                return Vec::new();
            }
        }

        let threshold_secs = (self.threshold_days as i64).saturating_mul(SECONDS_PER_DAY);
        let cutoff = self.now_ts.saturating_sub(threshold_secs);

        scraps
            .par_iter()
            .filter_map(|scrap| {
                let path = self.scraps_dir.join(scrap_relative_path(scrap));
                let ts = match self.git_command.commited_ts(&path) {
                    Ok(Some(ts)) => ts,
                    Ok(None) => return None,
                    Err(_) => return None,
                };
                if ts >= cutoff {
                    return None;
                }
                let age_days = (self.now_ts - ts) / SECONDS_PER_DAY;
                Some(LintWarning {
                    rule_name: LintRuleName::StaleByGit,
                    scrap_path: scrap_relative_path(scrap),
                    message: format!("scrap not updated in {} days", age_days),
                    source: None,
                    span: None,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraps_libs::git::tests::GitCommandTest;
    use std::io;
    use std::path::Path;

    /// Stub git command that returns scripted timestamps and repo status.
    #[derive(Clone, Copy)]
    struct GitStub {
        ts: Option<i64>,
        is_repo: bool,
    }

    impl GitCommand for GitStub {
        fn init(&self, _path: &Path) -> io::Result<()> {
            Ok(())
        }
        fn commited_ts(&self, _path: &Path) -> io::Result<Option<i64>> {
            Ok(self.ts)
        }
        fn is_git_repository(&self, _path: &Path) -> io::Result<bool> {
            Ok(self.is_repo)
        }
    }

    fn now_ts() -> i64 {
        1_700_000_000
    }

    #[test]
    fn flag_scrap_older_than_threshold() {
        let now = now_ts();
        let old_ts = now - 200 * SECONDS_PER_DAY;
        let rule = StaleByGitRule {
            git_command: GitStub {
                ts: Some(old_ts),
                is_repo: true,
            },
            scraps_dir: PathBuf::from("/tmp"),
            threshold_days: 180,
            now_ts: now,
        };
        let scraps = vec![Scrap::new("old", &None, "body")];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = rule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].rule_name, LintRuleName::StaleByGit);
        assert_eq!(warnings[0].scrap_path, "old.md");
        assert!(warnings[0].message.contains("200 days"));
    }

    #[test]
    fn skip_scrap_within_threshold() {
        let now = now_ts();
        let recent_ts = now - 30 * SECONDS_PER_DAY;
        let rule = StaleByGitRule {
            git_command: GitStub {
                ts: Some(recent_ts),
                is_repo: true,
            },
            scraps_dir: PathBuf::from("/tmp"),
            threshold_days: 180,
            now_ts: now,
        };
        let scraps = vec![Scrap::new("recent", &None, "body")];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = rule.check(&scraps, &backlinks_map, &tags);
        assert!(warnings.is_empty());
    }

    #[test]
    fn skip_uncommitted_scrap() {
        let now = now_ts();
        let rule = StaleByGitRule {
            git_command: GitStub {
                ts: None,
                is_repo: true,
            },
            scraps_dir: PathBuf::from("/tmp"),
            threshold_days: 180,
            now_ts: now,
        };
        let scraps = vec![Scrap::new("brand_new", &None, "body")];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = rule.check(&scraps, &backlinks_map, &tags);
        assert!(warnings.is_empty());
    }

    #[test]
    fn skip_when_not_a_git_repo() {
        let now = now_ts();
        let rule = StaleByGitRule {
            git_command: GitStub {
                ts: Some(0),
                is_repo: false,
            },
            scraps_dir: PathBuf::from("/tmp"),
            threshold_days: 180,
            now_ts: now,
        };
        let scraps = vec![Scrap::new("a", &None, "body")];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = rule.check(&scraps, &backlinks_map, &tags);
        assert!(warnings.is_empty());
    }

    #[test]
    fn flag_scrap_with_context_path() {
        let now = now_ts();
        let old_ts = now - 365 * SECONDS_PER_DAY;
        let rule = StaleByGitRule {
            git_command: GitStub {
                ts: Some(old_ts),
                is_repo: true,
            },
            scraps_dir: PathBuf::from("/tmp"),
            threshold_days: 180,
            now_ts: now,
        };
        let scraps = vec![Scrap::new("note", &Some("ai".into()), "body")];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = rule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].scrap_path, "ai/note.md");
    }

    #[test]
    fn default_test_git_returns_zero_so_old_scraps_flagged() {
        // Sanity check that scraps_libs GitCommandTest still works as a stub.
        let rule = StaleByGitRule {
            git_command: GitCommandTest::new(),
            scraps_dir: PathBuf::from("/tmp"),
            threshold_days: 180,
            now_ts: now_ts(),
        };
        let scraps = vec![Scrap::new("a", &None, "")];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = rule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 1);
    }
}
