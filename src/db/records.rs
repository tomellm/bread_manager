use bincode as bc;
use chrono::{DateTime, Local};
use futures::executor::Enter;
use hermes::impl_to_active_model;
use sea_orm::{entity::prelude::*, EntityOrSelect, JoinType, QuerySelect};
use sea_query::{Alias, ExprTrait};
use sqlx_projector::projectors::{FromEntity, ToEntity};
use uuid::Uuid;

use crate::model::records::{ExpenseRecord, ExpenseRecordState};

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
    state: String,
}

impl From<ExpenseRecordState> for sea_query::Value {
    fn from(value: ExpenseRecordState) -> Self {
        value.to_string().into()
    }
}

pub(crate) type DbRecord = Entity;

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    DataImport,
    NegativePossibleLink,
    PositivePossibleLink,
    NegativeLink,
    PositiveLink,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Relation::DataImport => {
                Entity::belongs_to(super::data_import::Entity)
                    .from(Column::DataImport)
                    .to(super::data_import::Column::Uuid)
                    .into()
            }
            Relation::NegativePossibleLink => Entity::leading_poss_link_rel(),
            Relation::PositivePossibleLink => Entity::following_poss_link_rel(),
            Relation::NegativeLink => Entity::leading_link_rel(),
            Relation::PositiveLink => Entity::following_link_rel(),
        }
    }
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
            description_container: bc::serialize(
                entity.description_container(),
            )
            .unwrap(),
            tags: entity.tags().clone().join(TAG_SEPARATOR),
            data: bc::serialize(entity.data()).unwrap(),
            origin: entity.origin().clone(),
            data_import: *entity.data_import(),
            state: entity.state().to_string(),
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
            self.state.into(),
        )
    }
}

impl_to_active_model!(ExpenseRecord, Model);

impl Entity {
    pub fn find_all_active() -> Select<Self> {
        Self::find()
            .select()
            .join_as(
                JoinType::LeftJoin,
                Self::following_link_rel(),
                Alias::new("pos"),
            )
            .join_as(
                JoinType::LeftJoin,
                Self::leading_link_rel(),
                Alias::new("neg"),
            )
            .filter(
                Column::State
                    .eq(ExpenseRecordState::Active)
                    .and(
                        Expr::col((
                            Alias::new("pos"),
                            super::link::Column::Leading,
                        ))
                        .is_null(),
                    )
                    .and(
                        Expr::col((
                            Alias::new("neg"),
                            super::link::Column::Following,
                        ))
                        .is_null(),
                    ),
            )
    }

    pub fn leading_link_rel() -> RelationDef {
        Entity::belongs_to(super::link::Entity)
            .from(Column::Uuid)
            .to(super::link::Column::Leading)
            .into()
    }
    pub fn following_link_rel() -> RelationDef {
        Entity::belongs_to(super::link::Entity)
            .from(Column::Uuid)
            .to(super::link::Column::Following)
            .into()
    }

    pub fn leading_poss_link_rel() -> RelationDef {
        Entity::belongs_to(super::possible_links::Entity)
            .from(Column::Uuid)
            .to(super::possible_links::Column::Leading)
            .into()
    }

    pub fn following_poss_link_rel() -> RelationDef {
        Entity::belongs_to(super::possible_links::Entity)
            .from(Column::Uuid)
            .to(super::possible_links::Column::Following)
            .into()
    }
}
