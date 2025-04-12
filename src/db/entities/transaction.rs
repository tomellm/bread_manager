use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "transaction")]
pub struct Model {
    #[sea_orm(primary_key)]
    uuid: Uuid,
    state: State,
}

#[derive(EnumIter, DeriveActiveEnum, Clone, Debug, PartialEq, Eq)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(255))")]
pub enum State {
    #[sea_orm(string_value = "Active")]
    Active,
    #[sea_orm(string_value = "Deleted")]
    Deleted,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
