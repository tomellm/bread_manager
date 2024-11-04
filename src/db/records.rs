use std::sync::Arc;

use bincode as bc;
use chrono::{DateTime, Local};
use diesel::{Insertable, Queryable, Selectable};

use crate::{model::records::ExpenseRecord, schema};

use super::Uuid;

const TAG_SEPARATOR: &str = ";";

pub static RECORDS_FROM_DB_FN: Arc<dyn Fn(DbRecord) -> ExpenseRecord + Sync + Send + 'static> =
    Arc::new(|val: DbRecord| val.into_record());

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = schema::expense_records)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub(crate) struct DbRecord {
    datetime_created: DateTime<Local>,
    uuid: Uuid,
    amount: i32,
    datetime: DateTime<Local>,
    description: Option<String>,
    description_container: Vec<u8>,
    tags: String,
    origin: String,
    data: Vec<u8>,
}

impl DbRecord {
    pub fn from_record(record: &ExpenseRecord) -> Self {
        Self {
            datetime_created: record.created().clone().timestamp(),
            uuid: **record.uuid(),
            amount: *record.amount() as i32,
            datetime: record.datetime().clone().timestamp(),
            description: record.description().cloned(),
            description_container: bc::serialize(record.description_container()).unwrap(),
            tags: record.tags().clone().join(TAG_SEPARATOR),
            data: bc::serialize(record.data()).unwrap(),
            origin: record.origin().clone(),
        }
    }

    pub fn into_record(self) -> ExpenseRecord {
        ExpenseRecord::new_all(
            DateTime::from_timestamp(self.datetime_created, 0)
                .map(|d| d.with_timezone(&Local::now().timezone()))
                .unwrap(),
            self.uuid,
            isize::try_from(self.amount).unwrap(),
            DateTime::from_timestamp(self.datetime, 0)
                .map(|d| d.with_timezone(&Local::now().timezone()))
                .unwrap(),
            bc::deserialize(&self.description_container).unwrap(),
            bc::deserialize(&self.data).unwrap(),
            self.tags
                .split(TAG_SEPARATOR)
                .map(str::to_string)
                .collect::<Vec<String>>(),
            self.origin,
        )
    }
}
