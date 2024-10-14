use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};

use crate::model::{profiles::error::ProfileError, records::ExpenseData};

use super::{ParsableWrapper, Parser};

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ExpenseDateTime(pub String);

impl From<ExpenseDateTime> for ParsableWrapper {
    fn from(value: ExpenseDateTime) -> Self {
        Self::ExpenseDateTime(value)
    }
}

impl Parser<DateTime<Local>> for ExpenseDateTime {
    fn parse_str(&self, str: &str) -> Result<DateTime<Local>, ProfileError> {
        NaiveDateTime::parse_from_str(str, &self.0)
            .map(|dt| dt.and_local_timezone(Local::now().timezone()).unwrap())
            .or(Err(ProfileError::date(str, &self.0)))
    }
    fn to_expense_data(&self, str: &str) -> Result<ExpenseData, ProfileError> {
        Ok(ExpenseData::ExpenseDateTime(self.parse_str(str)?))
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ExpenseDate(pub String);

impl From<ExpenseDate> for ParsableWrapper {
    fn from(value: ExpenseDate) -> Self {
        Self::ExpenseDate(value)
    }
}

impl Parser<NaiveDate> for ExpenseDate {
    fn parse_str(&self, str: &str) -> Result<NaiveDate, ProfileError> {
        NaiveDate::parse_from_str(str, &self.0).or(Err(ProfileError::date(str, &self.0)))
    }
    fn to_expense_data(&self, str: &str) -> Result<ExpenseData, ProfileError> {
        Ok(ExpenseData::ExpenseDate(self.parse_str(str)?))
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ExpenseTime(pub String);

impl From<ExpenseTime> for ParsableWrapper {
    fn from(value: ExpenseTime) -> Self {
        Self::ExpenseTime(value)
    }
}

impl Parser<NaiveTime> for ExpenseTime {
    fn parse_str(&self, str: &str) -> Result<NaiveTime, ProfileError> {
        NaiveTime::parse_from_str(str, &self.0).or(Err(ProfileError::date(str, &self.0)))
    }
    fn to_expense_data(&self, str: &str) -> Result<ExpenseData, ProfileError> {
        Ok(ExpenseData::ExpenseTime(self.parse_str(str)?))
    }
}
