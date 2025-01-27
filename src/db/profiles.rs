use hermes::impl_to_active_model;
use sea_orm::entity::prelude::*;
use sqlx_projector::projectors::{FromEntity, ToEntity};
use uuid::Uuid;

use crate::model::profiles::Profile;

#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "profiles")]
pub struct Model {
    #[sea_orm(primary_key)]
    uuid: Uuid,
    name: String,
    origin_name: String,
    data: Vec<u8>,
}

pub(crate) type DbProfile = Entity;

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl FromEntity<Profile> for Model {
    fn from_entity(entity: Profile) -> Self {
        let (uuid, name, origin_name, data) = entity.to_db();
        Self {
            uuid,
            name,
            origin_name,
            data,
        }
    }
}

impl ToEntity<Profile> for Model {
    fn to_entity(self) -> Profile {
        Profile::from_db(self.uuid, self.name, self.origin_name, &self.data)
    }
}

impl_to_active_model!(Profile, Model);

//create table if not exists profiles (
//    uuid blob primary key not null,
//    name text not null,
//    origin_name text not null,
//    data blob not null
//);
