use hermes::impl_to_active_model;
use sea_orm::entity::prelude::*;
use sea_orm::DeriveEntityModel;
use sqlx_projector::projectors::{FromEntity, ToEntity};
use uuid::Uuid;

use crate::model::linker::PossibleLink;

#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "possible_links")]
pub struct Model {
    #[sea_orm(primary_key)]
    uuid: Uuid,
    negative: Uuid,
    positive: Uuid,
    probability: f64,
}

pub type DbPossibleLink = Entity;

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl FromEntity<PossibleLink> for Model {
    fn from_entity(entity: PossibleLink) -> Self {
        Self {
            uuid: entity.uuid,
            negative: *entity.negative,
            positive: *entity.positive,
            probability: entity.probability,
        }
    }
}

impl ToEntity<PossibleLink> for Model {
    fn to_entity(self) -> PossibleLink {
        PossibleLink {
            uuid: self.uuid,
            negative: self.negative.into(),
            positive: self.positive.into(),
            probability: self.probability,
        }
    }
}

impl_to_active_model!(PossibleLink, Model);

//create table if not exists possible_links (
//    uuid blob primary key not null,
//    negative blob not null,
//    positive blob not null,
//    probability real not null
//);
