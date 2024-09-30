use std::sync::Arc;

use bincode as bc;
use chrono::{DateTime, Local};
use data_communicator::buffered::{
    query::{self, QueryResponse},
    storage::{self, Storage},
    GetKey,
};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

const TAG_SEP: &str = ";";

use crate::model::records::ExpenseRecord;

use super::{utils, IntoChangeResult, MapToQueryError};

#[derive(sqlx::FromRow)]
struct DbRecord {
    datetime_created: i64,
    uuid: Uuid,
    amount: i64,
    datetime: i64,
    description: Option<String>,
    description_container: Vec<u8>,
    tags: String,
    origin: String,
    data: Vec<u8>,
}

impl GetKey<Uuid> for ExpenseRecord {
    fn key(&self) -> &Uuid {
        self.uuid()
    }
}
impl DbRecord {
    pub fn from_record(record: &ExpenseRecord) -> Self {
        Self {
            datetime_created: record.created().clone().timestamp(),
            uuid: **record.uuid(),
            amount: *record.amount() as i64,
            datetime: record.datetime().clone().timestamp(),
            description: record.description().cloned(),
            description_container: bc::serialize(record.description_container()).unwrap(),
            tags: record.tags().clone().join(TAG_SEP),
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
                .split(TAG_SEP)
                .map(str::to_string)
                .collect::<Vec<String>>(),
            self.origin,
        )
    }
}

pub struct DbRecords {
    pool: Arc<Pool<Sqlite>>,
}

impl DbRecords {
    pub fn pool(&self) -> Arc<Pool<Sqlite>> {
        self.pool.clone()
    }
}

pub struct DbRecordsInitArgs {
    pub pool: Arc<Pool<Sqlite>>,
    pub drop: bool,
}

impl From<(&Arc<Pool<Sqlite>>, bool)> for DbRecordsInitArgs {
    fn from(value: (&Arc<Pool<Sqlite>>, bool)) -> Self {
        Self { pool: value.0.clone(), drop: value.1 }
    }
}

impl Storage<Uuid, ExpenseRecord> for DbRecords {
    type InitArgs = DbRecordsInitArgs;
    fn init(DbRecordsInitArgs { pool, drop }: Self::InitArgs) -> impl storage::InitFuture<Self> {
        async move {
            if drop {
                let _ = sqlx::query!("drop table if exists expense_records;")
                    .execute(&*pool)
                    .await
                    .unwrap();
            }

            let _ = sqlx::query!(
                r#"
                create table if not exists expense_records (
                    datetime_created integer not null,
                    uuid blob primary key not null,
                    amount integer not null,
                    datetime integer not null,
                    description text,
                    description_container blob not null,
                    tags text not null,
                    origin text not null,
                    data blob not null
                );
                "#
            )
            .execute(&*pool)
            .await
            .unwrap();
            Self { pool }
        }
    }
    fn update(
        &mut self,
        value: &ExpenseRecord,
    ) -> impl storage::Future<data_communicator::buffered::change::ChangeResult> {
        let pool = self.pool();
        let record = DbRecord::from_record(value);
        async move {
            sqlx::query!(
                r#"insert into expense_records(
                    datetime_created, uuid, amount, datetime, description, description_container, tags, origin, data
                ) values(?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
                record.datetime_created,
                record.uuid,
                record.amount,
                record.datetime,
                record.description,
                record.description_container,
                record.tags,
                record.origin,
                record.data,
            )
                .execute(&*pool)
                .await
                .into_change_result()
        }
    }
    fn update_many(
        &mut self,
        values: &[ExpenseRecord],
    ) -> impl storage::Future<data_communicator::buffered::change::ChangeResult> {
        let pool = self.pool();
        let records = values.iter().map(DbRecord::from_record).collect::<Vec<_>>();
        async move {
            utils::insert_values(
                pool,
                "insert into expense_records(
                    datetime_created, uuid, amount, datetime, description,
                    description_container, tags, origin, data
                )",
                records,
                |mut builder, value| {
                    builder
                        .push_bind(value.datetime_created)
                        .push_bind(value.uuid)
                        .push_bind(value.amount)
                        .push_bind(value.datetime)
                        .push_bind(value.description)
                        .push_bind(value.description_container)
                        .push_bind(value.tags)
                        .push_bind(value.origin)
                        .push_bind(value.data);
                },
            )
            .await
            .into_change_result()
        }
    }
    fn delete(
        &mut self,
        key: &Uuid,
    ) -> impl storage::Future<data_communicator::buffered::change::ChangeResult> {
        let pool = self.pool();
        let key = *key;
        async move {
            sqlx::query!("delete from expense_records where uuid = ?", key)
                .execute(&*pool)
                .await
                .into_change_result()
        }
    }
    fn delete_many(
        &mut self,
        keys: &[Uuid],
    ) -> impl storage::Future<data_communicator::buffered::change::ChangeResult> {
        let pool = self.pool();
        let keys = keys.to_vec();
        async move {
            utils::add_in_items(
                "delete from expense_records where uuid in (",
                keys.into_iter(),
                ")",
            )
            .build()
            .execute(&*pool)
            .await
            .into_change_result()
        }
    }
    fn get_all(&mut self) -> impl storage::Future<QueryResponse<Uuid, ExpenseRecord>> {
        let pool = self.pool();
        async move {
            sqlx::query_as!(
                DbRecord,
                r#"
                select 
                    datetime_created, uuid as "uuid: uuid::Uuid", amount, 
                    datetime, description, description_container, tags, origin,
                    data
                from
                    expense_records
                "#
            )
            .fetch_all(&*pool)
            .await
            .map(|vals| {
                vals.into_iter()
                    .map(DbRecord::into_record)
                    .collect::<Vec<_>>()
            })
            .map_query_error()
            .into()
        }
    }
    fn get_by_id(&mut self, key: Uuid) -> impl storage::Future<QueryResponse<Uuid, ExpenseRecord>> {
        let pool = self.pool();
        async move {
            sqlx::query_as!(
                DbRecord,
                r#"
                select
                    datetime_created, uuid as "uuid: uuid::Uuid", amount, 
                    datetime, description, description_container, tags, origin,
                    data
                from
                    expense_records
                where
                    uuid = ?
                "#,
                key
            )
            .fetch_one(&*pool)
            .await
            .map(DbRecord::into_record)
            .map_query_error()
            .into()
        }
    }
    fn get_by_ids(
        &mut self,
        keys: Vec<Uuid>,
    ) -> impl storage::Future<QueryResponse<Uuid, ExpenseRecord>> {
        let pool = self.pool();
        async move {
            utils::add_in_items(
                r#"select
                    datetime_created, uuid as "uuid: uuid::Uuid", amount, 
                    datetime, description, description_container, tags, origin,
                    data
                from
                    expense_records
                where
                    uuid in ("#,
                keys.iter(),
                ")",
            )
            .build_query_as::<DbRecord>()
            .fetch_all(&*pool)
            .await
            .map(|vals| {
                vals.into_iter()
                    .map(DbRecord::into_record)
                    .collect::<Vec<_>>()
            })
            .map_query_error()
            .into()
        }
    }
    fn get_by_predicate(
        &mut self,
        predicate: query::Predicate<ExpenseRecord>,
    ) -> impl storage::Future<QueryResponse<Uuid, ExpenseRecord>> {
        let pool = self.pool();
        async move {
            sqlx::query_as!(
                DbRecord,
                r#"
                select 
                    datetime_created, uuid as "uuid: uuid::Uuid", amount, 
                    datetime, description, description_container, tags, origin,
                    data
                from
                    expense_records
                "#
            )
            .fetch_all(&*pool)
            .await
            .map(|values| {
                values
                    .into_iter()
                    .filter_map(|val| {
                        let record = val.into_record();
                        if predicate(&record) {
                            Some(record)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .map_query_error()
            .into()
        }
    }
}
