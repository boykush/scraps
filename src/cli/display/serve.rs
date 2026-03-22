use std::fmt;

use colored::Colorize;

pub struct DisplayServeInfo {
    title: String,
    url: String,
    scrap_count: usize,
}

impl DisplayServeInfo {
    pub fn new(title: &str, url: &str, scrap_count: usize) -> Self {
        Self {
            title: title.to_string(),
            url: url.to_string(),
            scrap_count,
        }
    }
}

impl fmt::Display for DisplayServeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lines = [
            String::new(),
            format!("{}  {}", "Serving:".bold(), self.title),
            format!("{}    {}", "Local:".bold(), self.url.cyan()),
            format!(
                "{}   {}",
                "Scraps:".bold(),
                format!("{} pages", self.scrap_count)
            ),
            String::new(),
            format!("Press {} to stop", "Ctrl+C".bold()),
            String::new(),
        ];

        // Calculate max visible width (without ANSI escape codes)
        let padding = 3;
        let visible_widths: Vec<usize> = lines.iter().map(|line| strip_ansi_width(line)).collect();
        let max_width = visible_widths.iter().copied().max().unwrap_or(0) + padding * 2;

        // Draw box
        writeln!(f)?;
        writeln!(f, "  \u{250c}{}\u{2510}", "\u{2500}".repeat(max_width))?;
        for (line, visible_width) in lines.iter().zip(visible_widths.iter()) {
            let right_pad = max_width - padding - visible_width;
            writeln!(
                f,
                "  \u{2502}{}{}{}\u{2502}",
                " ".repeat(padding),
                line,
                " ".repeat(right_pad)
            )?;
        }
        write!(f, "  \u{2514}{}\u{2518}", "\u{2500}".repeat(max_width))
    }
}

fn strip_ansi_width(s: &str) -> usize {
    let mut width = 0;
    let mut in_escape = false;
    for c in s.chars() {
        if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else if c == '\x1b' {
            in_escape = true;
        } else {
            width += 1;
        }
    }
    width
}
