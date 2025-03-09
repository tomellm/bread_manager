use hermes::impl_to_active_model;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelBehavior, DeriveEntityModel, DeriveRelation, EntityOrSelect, EnumIter};
use sqlx_projector::projectors::{FromEntity, ToEntity};
use uuid::Uuid;

use crate::model::linker::Link;

#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "links")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub uuid: Uuid,
    pub negative: Uuid,
    pub positive: Uuid,
    pub deleted: bool,
}

pub(crate) type DbLink = Entity;

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl FromEntity<Link> for Model {
    fn from_entity(entity: Link) -> Self {
        Self {
            uuid: entity.uuid,
            negative: *entity.negative,
            positive: *entity.positive,
            deleted: entity.deleted,
        }
    }
}

impl ToEntity<Link> for Model {
    fn to_entity(self) -> Link {
        Link {
            uuid: self.uuid,
            negative: self.negative.into(),
            positive: self.positive.into(),
            deleted: self.deleted,
        }
    }
}

impl_to_active_model!(Link, Model);

impl Entity {
    pub fn find_all_active() -> Select<Self> {
        Self::find().select().filter(Column::Deleted.eq(false))
    }
}

//create table if not exists links (
//    uuid blob primary key not null,
//    negative blob not null,
//    positive blob not null
//);
