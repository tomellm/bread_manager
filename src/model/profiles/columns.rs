use error::ProfileError;
use std::collections::{HashMap, HashSet};
use tracing::trace;

use bincode as bc;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

use crate::model::records::ExpenseData;

use super::error::ProfileError;

fn to_num(str: &str) -> Result<f64, ProfileError> {
    str.replace('.', "")
        .replace(',', ".")
        .parse::<f64>()
        .or(Err(ProfileError::number(str, "f64")))
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Income;
impl Income {
    pub fn parse_str(str: &str) -> Result<usize, ProfileError> {
        if str.is_empty() {
            return Ok(0);
        }
        Ok((to_num(str)? * 100.0) as usize)
    }
}
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Expense;
impl Expense {
    pub fn parse_str(str: &str) -> Result<usize, ProfileError> {
        if str.is_empty() {
            return Ok(0);
        }
        Ok((to_num(str)? * -100.0) as usize)
    }
}
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PosExpense;
impl PosExpense {
    pub fn parse_str(str: &str) -> Result<usize, ProfileError> {
        if str.is_empty() {
            return Ok(0);
        }
        Ok((to_num(str)? * 100.0) as usize)
    }
}
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Movement;
impl Movement {
    pub fn parse_str(str: &str) -> Result<isize, ProfileError> {
        if str.is_empty() {
            return Ok(0);
        }
        Ok((to_num(str)? * 100.0) as isize)
    }
}
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ExpenseDateTime(pub String);
impl ExpenseDateTime {
    pub fn parse_str(&self, str: &str) -> Result<DateTime<Local>, ProfileError> {
        NaiveDateTime::parse_from_str(str, &self.0)
            .map(|dt| dt.and_local_timezone(Local::now().timezone()).unwrap())
            .or(Err(ProfileError::date(str, &self.0)))
    }
}
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ExpenseDate(pub String);
impl ExpenseDate {
    pub fn parse_str(&self, str: &str) -> Result<NaiveDate, ProfileError> {
        NaiveDate::parse_from_str(str, &self.0).or(Err(ProfileError::date(str, &self.0)))
    }
}
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ExpenseTime(pub String);
impl ExpenseTime {
    pub fn parse_str(&self, str: &str) -> Result<NaiveTime, ProfileError> {
        NaiveTime::parse_from_str(str, &self.0).or(Err(ProfileError::date(str, &self.0)))
    }
}
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Description(pub String);

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Other(pub String);

pub trait Parsable {
    fn to_expense_data(&self, str: String) -> Result<ExpenseData, ProfileError>;
}
