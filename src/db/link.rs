use hermes::impl_to_active_model;
use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelBehavior, DeriveEntityModel, EntityOrSelect, EnumIter,
};
use sqlx_projector::impl_to_database;
use sqlx_projector::projectors::{FromEntity, ToEntity};
use uuid::Uuid;

use crate::model::linker::{Link, LinkType};

#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "links")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub uuid: Uuid,
    pub leading: Uuid,
    pub following: Uuid,
    pub deleted: bool,
    pub link_type: String,
}

impl From<LinkType> for sea_query::Value {
    fn from(value: LinkType) -> Self {
        value.to_string().into()
    }
}

impl From<String> for LinkType {
    fn from(value: String) -> Self {
        // ToDo: instead of writing out the strings I could
        // use a list of the values and compare using to_string
        match value.as_str() {
            "Transfer" => Self::Transfer,
            "DuplicateOf" => Self::DuplicateOf,
            _ => unreachable!(),
        }
    }
}

impl_to_database!(Link, <DbLink as EntityTrait>::Model);

pub(crate) type DbLink = Entity;

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl FromEntity<Link> for Model {
    fn from_entity(entity: Link) -> Self {
        Self {
            uuid: entity.uuid,
            leading: *entity.leading,
            following: *entity.following,
            deleted: entity.deleted,
            link_type: entity.link_type.to_string(),
        }
    }
}

impl ToEntity<Link> for Model {
    fn to_entity(self) -> Link {
        Link {
            uuid: self.uuid,
            leading: self.leading.into(),
            following: self.following.into(),
            deleted: self.deleted,
            link_type: self.link_type.into(),
        }
    }
}

impl_to_active_model!(Link, Model);

impl Entity {
    pub fn find_all_active() -> Select<Self> {
        Self::find().select().filter(Column::Deleted.eq(false))
    }

    pub fn leading_rel() -> RelationDef {
        Entity::belongs_to(super::records::Entity)
            .from(Column::Leading)
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

//create table if not exists links (
//    uuid blob primary key not null,
//    negative blob not null,
//    positive blob not null
//);
