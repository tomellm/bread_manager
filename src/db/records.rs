use bincode as bc;
use chrono::{DateTime, Local};
use hermes::impl_to_active_model;
use sea_orm::entity::prelude::*;
use sqlx_projector::projectors::{FromEntity, ToEntity};
use uuid::Uuid;

use crate::model::records::ExpenseRecord;

const TAG_SEPARATOR: &str = ";";

#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "expense_records")]
pub struct Model {
    #[sea_orm(primary_key)]
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

pub(crate) type DbRecord = Entity;

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl FromEntity<ExpenseRecord> for Model {
    fn from_entity(entity: ExpenseRecord) -> Self {
        Self {
            datetime_created: entity.created().clone().timestamp(),
            uuid: **entity.uuid(),
            amount: *entity.amount() as i64,
            datetime: entity.datetime().clone().timestamp(),
            description: entity.description().cloned(),
            description_container: bc::serialize(entity.description_container()).unwrap(),
            tags: entity.tags().clone().join(TAG_SEPARATOR),
            data: bc::serialize(entity.data()).unwrap(),
            origin: entity.origin().clone(),
        }
    }
}

impl ToEntity<ExpenseRecord> for Model {
    fn to_entity(self) -> ExpenseRecord {
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

impl_to_active_model!(ExpenseRecord, Model);

//r#"
//create table if not exists expense_records (
//    datetime_created integer not null,
//    uuid blob primary key not null,
//    amount integer not null,
//    datetime integer not null,
//    description text,
//    description_container blob not null,
//    tags text not null,
//    origin text not null,
//    data blob not null
//);
