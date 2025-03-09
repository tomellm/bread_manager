use bincode as bc;
use chrono::{DateTime, Local};
use hermes::impl_to_active_model;
use sea_orm::{entity::prelude::*, EntityOrSelect};
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
    data_import: Uuid,
    data: Vec<u8>,
    deleted: bool,
}

pub(crate) type DbRecord = Entity;

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::data_import::Entity",
        from = "Column::DataImport",
        to = "super::data_import::Column::Uuid"
    )]
    DataImport,
}

impl Related<super::data_import::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DataImport.def()
    }
}

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
            data_import: *entity.data_import(),
            deleted: entity.deleted(),
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
            self.data_import,
            self.deleted,
        )
    }
}

impl_to_active_model!(ExpenseRecord, Model);

impl Entity {
    pub fn find_all_active() -> Select<Self> {
        Self::find().select().filter(Column::Deleted.eq(false))
    }
}
