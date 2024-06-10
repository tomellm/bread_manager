use std::sync::Arc;

use bincode as bc;
use chrono::{DateTime, Local};
use futures::future::BoxFuture;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

const TAG_SEP: &str = ";";

use crate::{
    model::records::ExpenseRecord,
    utils::{
        changer::{self, ActionType, Response},
        communicator::{GetKey, Storage},
    },
};

use super::{error_to_response, utils};

pub struct DbRecords {
    pub(super) pool: Arc<Pool<Sqlite>>,
}

struct DbRecord {
    datetime_created: i64,
    uuid: Uuid,
    amount: i64,
    datetime: i64,
    description: Option<String>,
    description_container: Vec<u8>,
    tags: String,
    data: Vec<u8>,
}

impl GetKey<Uuid> for ExpenseRecord {
    fn get_key(&self) -> Uuid {
        **self.uuid()
    }
}
impl DbRecord {
    pub fn from_record(record: &ExpenseRecord) -> Self {
        Self {
            datetime_created: record.created().clone().timestamp(),
            uuid: *record.uuid().clone(),
            amount: *record.amount() as i64,
            datetime: record.datetime().clone().timestamp(),
            description: record.description().cloned(),
            description_container: bc::serialize(record.description_container()).unwrap(),
            tags: record.tags().clone().join(TAG_SEP),
            data: bc::serialize(record.data()).unwrap(),
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
        )
    }
}

impl Storage<Uuid, ExpenseRecord> for DbRecords {
    fn get_all(&self) -> BoxFuture<'static, changer::Response<Uuid, ExpenseRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let records = sqlx::query_as!(
                DbRecord,
                r#"
                select 
                    datetime_created,
                    uuid as "uuid: uuid::Uuid",
                    amount,
                    datetime,
                    description,
                    description_container,
                    tags,
                    data
                from
                    expense_records
                "#
            )
            .fetch_all(&*pool)
            .await
            .unwrap()
            .into_iter()
            .map(DbRecord::into_record)
            .collect();

            let action = ActionType::GetAll(records);
            Response::ok(&action)
        })
    }
    fn set(
        &self,
        value: ExpenseRecord,
    ) -> BoxFuture<'static, changer::Response<Uuid, ExpenseRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let record = DbRecord::from_record(&value);
            let query_result = sqlx::query!(
                "insert into expense_records values(?, ?, ?, ?, ?, ?, ?, ?)",
                record.datetime_created,
                record.uuid,
                record.amount,
                record.datetime,
                record.description,
                record.description_container,
                record.tags,
                record.data
            )
            .execute(&*pool)
            .await;
            error_to_response(query_result, &ActionType::Set(value))
        })
    }
    fn set_many(
        &self,
        values: Vec<ExpenseRecord>,
    ) -> BoxFuture<'static, Response<Uuid, ExpenseRecord>> {
        println!("setting many right now");
        let pool = self.pool.clone();
        Box::pin(async move {
            let records = values.iter().map(DbRecord::from_record).collect::<Vec<_>>();

            let query_result = utils::insert_values(
                pool,
                "insert into expense_records(
                    datetime_created, uuid, amount, datetime, description, 
                    description_container, tags, data
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
                        .push_bind(value.data);
                },
            )
            .await;

            Response::from_result(query_result, &ActionType::SetMany(values))
        })
    }
    fn update(
        &self,
        value: ExpenseRecord,
    ) -> BoxFuture<'static, changer::Response<Uuid, ExpenseRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let record = DbRecord::from_record(&value);
            let query_result = sqlx::query!(
                "insert into expense_records values(?, ?, ?, ?, ?, ?, ?, ?)",
                record.datetime_created,
                record.uuid,
                record.amount,
                record.datetime,
                record.description,
                record.description_container,
                record.tags,
                record.data
            )
            .execute(&*pool)
            .await;
            error_to_response(query_result, &ActionType::Update(value))
        })
    }
    fn update_many(
        &self,
        values: Vec<ExpenseRecord>,
    ) -> BoxFuture<'static, Response<Uuid, ExpenseRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let records = values.iter().map(DbRecord::from_record).collect::<Vec<_>>();

            let query_result = utils::insert_values(
                pool,
                "insert into expense_records(
                    datetime_created, uuid, amount, datetime, description, 
                    description_container, tags, data
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
                        .push_bind(value.data);
                },
            )
            .await;

            Response::from_result(query_result, &ActionType::SetMany(values))
        })
    }
    fn delete(&self, key: Uuid) -> BoxFuture<'static, changer::Response<Uuid, ExpenseRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let query_result = sqlx::query!("delete from expense_records where uuid = ?", key)
                .execute(&*pool)
                .await;
            error_to_response(query_result, &ActionType::Delete(key))
        })
    }
    fn delete_many(&self, keys: Vec<Uuid>) -> BoxFuture<'static, Response<Uuid, ExpenseRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let query_result = utils::add_in_items(
                "delete from expense_records where uuid in (",
                keys.iter(),
                ")",
            )
            .build()
            .execute(&*pool)
            .await;

            Response::from_result(query_result, &ActionType::DeleteMany(keys))
        })
    }
    fn setup(&self, drop: bool) -> BoxFuture<'static, Result<Vec<ExpenseRecord>, ()>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            if drop {
                let _ = sqlx::query!("drop table if exists expense_records;")
                    .execute(&*pool)
                    .await
                    .map_err(|_| ())?;
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
                    data blob not null
                );
                "#
            )
            .execute(&*pool)
            .await
            .map_err(|_| ())?;

            Ok(sqlx::query_as!(
                DbRecord,
                r#"
                select
                    datetime_created,
                    uuid as "uuid: uuid::Uuid",
                    amount,
                    datetime,
                    description,
                    description_container,
                    tags,
                    data
                from
                    expense_records
                "#
            )
            .fetch_all(&*pool)
            .await
            .unwrap()
            .into_iter()
            .map(DbRecord::into_record)
            .collect())
        })
    }
}
