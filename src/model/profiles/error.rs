use chrono::{DateTime, Local};

#[derive(Debug, Clone)]
pub enum ProfileError {
    NumberParsing(String),
    DateParsing(String),
    ColumnWidth(String),
    BuildRecord(String),
}

impl ProfileError {
    pub fn number(str: &str, n_type: &str) -> Self {
        Self::NumberParsing(format!(
            "Parsing this string: {str} to this type: {n_type} failed"
        ))
    }
    pub fn date(str: &str, format: &str) -> Self {
        Self::DateParsing(format!(
            "This format: {format} does not fit this date string: {str}"
        ))
    }
    pub fn width(expected: usize, actual: usize) -> Self {
        Self::ColumnWidth(format!(
            "The profile expects a minimum width of {expected} but got a width of {actual}"
        ))
    }
    pub fn build(amount: Option<isize>, date: Option<DateTime<Local>>) -> Self {
        Self::BuildRecord(format!(
            "One of these two is not present: {amount:?} {date:?}"
        ))
    }
}
