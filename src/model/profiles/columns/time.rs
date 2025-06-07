use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};

use crate::model::{
    group::GroupUuid,
    profiles::error::ProfileError,
    transactions::{
        datetime::ModelDatetime, properties::TransactionProperties,
    },
};

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

    fn to_property(
        &self,
        group_uuid: GroupUuid,
        str: &str,
    ) -> Result<TransactionProperties, ProfileError> {
        let datetime = self.parse_str(str)?;
        Ok(ModelDatetime::init_datetime(datetime, group_uuid).into())
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
        NaiveDate::parse_from_str(str, &self.0)
            .or(Err(ProfileError::date(str, &self.0)))
    }

    fn to_property(
        &self,
        group_uuid: GroupUuid,
        str: &str,
    ) -> Result<TransactionProperties, ProfileError> {
        let date = self.parse_str(str)?;
        Ok(ModelDatetime::init(date, None, 3600, group_uuid).into())
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
        NaiveTime::parse_from_str(str, &self.0)
            .or(Err(ProfileError::date(str, &self.0)))
    }

    fn to_property(
        &self,
        _group_uuid: GroupUuid,
        _str: &str,
    ) -> Result<TransactionProperties, ProfileError> {
        // ToDo-[Thomas] - parse other time column as a special
        todo!()
    }
}
