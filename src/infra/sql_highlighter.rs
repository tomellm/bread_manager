#![cfg(feature = "color-sql")]

use syntect::{
    easy::HighlightLines,
    highlighting::{Style, Theme, ThemeSet},
    parsing::{SyntaxReference, SyntaxSet},
};

pub(crate) struct SqlHighlighter {
    syntax: SyntaxReference,
    syntax_set: SyntaxSet,
    theme: Theme,
}

impl SqlHighlighter {
    pub(crate) fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let syntax = syntax_set
            .find_syntax_by_extension("sql")
            .expect("SQL not defined")
            .clone();
        let theme = ThemeSet::load_defaults()
            .themes
            .get("base16-ocean.dark")
            .expect("Theme not found.")
            .clone();

        Self {
            syntax,
            syntax_set,
            theme,
        }
    }

    pub(crate) fn highlight_sql(&self, sql: &str) -> String {
        let mut h = HighlightLines::new(&self.syntax, &self.theme);
        sql.lines()
            .map(|line| {
                let ranges = h.highlight_line(line, &self.syntax_set);
                match ranges {
                    Ok(ranges) => as_terminal_escaped_no_bg(&ranges[..]),
                    Err(_) => line.to_string(),
                }
            })
            .collect::<Vec<_>>()
            .join("") //"\n")
    }
}

/// Formats the styled fragments using only foreground 24-bit color terminal escape codes.
fn as_terminal_escaped_no_bg(ranges: &[(Style, &str)]) -> String {
    ranges
        .iter()
        .map(|(style, text)| {
            let fg = style.foreground;
            format!("\x1b[38;2;{};{};{}m{}", fg.r, fg.g, fg.b, text)
        })
        .collect::<String>()
        + "\x1b[0m" // Reset colors at the end
}
