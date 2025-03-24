use hermes::impl_to_active_model;
use sea_orm::DeriveEntityModel;
use sea_orm::{entity::prelude::*, EntityOrSelect};
use sqlx_projector::impl_to_database;
use sqlx_projector::projectors::{FromEntity, ToEntity};
use uuid::Uuid;

use crate::model::linker::{PossibleLink, PossibleLinkState};

#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "possible_links")]
pub struct Model {
    #[sea_orm(primary_key)]
    uuid: Uuid,
    leading: Uuid,
    following: Uuid,
    probability: f64,
    state: String,
    link_type: String,
}

pub type DbPossibleLink = Entity;

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Leading,
    Following,
}

impl_to_database!(PossibleLink, <DbPossibleLink as EntityTrait>::Model);

impl From<PossibleLinkState> for sea_query::Value {
    fn from(value: PossibleLinkState) -> Self {
        value.to_string().into()
    }
}

impl From<String> for PossibleLinkState {
    fn from(value: String) -> Self {
        // ToDo: instead of writing out the strings I could
        // use a list of the values and compare using to_string
        match value.as_str() {
            "Active" => Self::Active,
            "Deleted" => Self::Deleted,
            "Converted" => Self::Converted,
            _ => unreachable!(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Leading => Entity::belongs_to(super::records::Entity)
                .from(Column::Leading)
                .to(super::records::Column::Uuid)
                .into(),
            Relation::Following => Entity::belongs_to(super::records::Entity)
                .from(Column::Following)
                .to(super::records::Column::Uuid)
                .into(),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl FromEntity<PossibleLink> for Model {
    fn from_entity(entity: PossibleLink) -> Self {
        Self {
            uuid: entity.uuid,
            leading: *entity.leading,
            following: *entity.following,
            probability: entity.probability,
            state: entity.state.to_string(),
            link_type: entity.link_type.to_string(),
        }
    }
}

impl ToEntity<PossibleLink> for Model {
    fn to_entity(self) -> PossibleLink {
        PossibleLink {
            uuid: self.uuid,
            leading: self.leading.into(),
            following: self.following.into(),
            probability: self.probability,
            state: self.state.into(),
            link_type: self.link_type.into(),
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

    pub fn leading_rel() -> RelationDef {
        Entity::belongs_to(super::records::Entity)
            .from(Column::Following)
            .to(super::records::Column::Uuid)
            .into()
    }
    pub fn following_rel() -> RelationDef {
        Entity::belongs_to(super::records::Entity)
            .from(Column::Following)
            .to(super::records::Column::Uuid)
            .into()
    }
}
