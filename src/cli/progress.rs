use std::time::Instant;

use colored::Colorize;

use crate::usecase::progress::{Progress, Stage};

/// 絵文字を使用した進捗表示の実装
#[derive(Debug)]
pub struct ProgressImpl {
    start: Instant,
}

impl ProgressImpl {
    pub fn init(start: Instant) -> Self {
        println!("{}", "📦 Building site...".bold());
        Self { start }
    }
}

impl Progress for ProgressImpl {
    fn start_stage(&self, stage: &Stage) {
        match stage {
            Stage::ReadScraps => {
                println!("{}", "→ Reading Markdown files");
            }
        }
    }

    fn complete_stage(&self, stage: &Stage, count: &Option<usize>) {
        match stage {
            Stage::ReadScraps => {
                if let Some(count) = count {
                    println!("✔ Find {} Scraps", count);
                }
            }
        }
    }

    fn end(&self) {
        let end = self.start.elapsed();
        println!(
            "{} {}.{} {}",
            "✨ Done build in".green(),
            end.as_secs().to_string().green(),
            end.subsec_millis().to_string().green(),
            "secs".green()
        );
    }
}
