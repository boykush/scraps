use std::time::{Duration, Instant};

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

use crate::usecase::progress::{Progress, Stage};

#[derive(Debug)]
pub struct ProgressImpl {
    start: Instant,
}

impl ProgressImpl {
    pub fn init(start: Instant) -> Self {
        println!("{}", "📦 Building site...".bold());
        Self { start }
    }

    fn spinner_style() -> ProgressStyle {
        ProgressStyle::default_spinner()
            .template("  {spinner:.green} {msg}")
            .expect("invalid template")
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
    }

    fn create_spinner(message: &str) -> ProgressBar {
        let bar = ProgressBar::new_spinner();
        bar.set_style(Self::spinner_style());
        bar.set_message(message.to_string());
        bar.enable_steady_tick(Duration::from_millis(80));
        bar
    }

    fn stage_message(stage: &Stage) -> &'static str {
        match stage {
            Stage::ReadScraps => "Reading Markdown files...",
            Stage::GenerateHtml => "Generating HTML files...",
            Stage::GenerateCss => "Generating CSS files...",
            Stage::GenerateJson => "Generating JSON files...",
        }
    }

    fn complete_message(stage: &Stage, count: &usize) -> String {
        match stage {
            Stage::ReadScraps => format!("Found {count} Scraps"),
            Stage::GenerateHtml => format!("Generated {count} HTML files"),
            Stage::GenerateCss => format!("Generated {count} CSS files"),
            Stage::GenerateJson => format!("Generated {count} JSON files"),
        }
    }
}

impl Progress for ProgressImpl {
    fn start_stage(&self, stage: &Stage) {
        let spinner = Self::create_spinner(Self::stage_message(stage));
        // Store spinner in thread-local so complete_stage can access it
        CURRENT_SPINNER.with(|cell| {
            cell.replace(Some(spinner));
        });
    }

    fn complete_stage(&self, stage: &Stage, count: &usize) {
        CURRENT_SPINNER.with(|cell| {
            if let Some(spinner) = cell.borrow_mut().take() {
                let msg = Self::complete_message(stage, count);
                spinner.finish_with_message(format!("  {} {}", "✔".green(), msg));
            }
        });
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

thread_local! {
    static CURRENT_SPINNER: std::cell::RefCell<Option<ProgressBar>> = const { std::cell::RefCell::new(None) };
}
