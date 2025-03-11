use hermes::impl_to_active_model;
use sea_orm::DeriveEntityModel;
use sea_orm::{entity::prelude::*, EntityOrSelect};
use sqlx_projector::projectors::{FromEntity, ToEntity};
use uuid::Uuid;

use crate::model::linker::{PossibleLink, PossibleLinkState};

#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "possible_links")]
pub struct Model {
    #[sea_orm(primary_key)]
    uuid: Uuid,
    negative: Uuid,
    positive: Uuid,
    probability: f64,
    state: String,
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
            state: entity.state.to_string(),
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
            state: self.state.into(),
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

impl Entity {
    pub fn find_all_active() -> Select<Self> {
        Self::find()
            .select()
            .filter(Column::State.eq(PossibleLinkState::Active))
    }
}
