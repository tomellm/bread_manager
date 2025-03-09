use hermes::impl_to_active_model;
use sea_orm::{entity::prelude::*, EntityOrSelect};
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
    deleted: bool,
}

pub(crate) type DbProfile = Entity;

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::data_import::Entity")]
    DataImport,
}

impl Related<super::data_import::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DataImport.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl FromEntity<Profile> for Model {
    fn from_entity(entity: Profile) -> Self {
        let (uuid, name, origin_name, data, deleted) = entity.to_db();
        Self {
            uuid,
            name,
            origin_name,
            data,
            deleted,
        }
    }
}

impl ToEntity<Profile> for Model {
    fn to_entity(self) -> Profile {
        Profile::from_db(
            self.uuid,
            self.name,
            self.origin_name,
            &self.data,
            self.deleted,
        )
    }
}

impl_to_active_model!(Profile, Model);

impl Entity {
    pub fn find_all_active() -> Select<Self> {
        Self::find().select().filter(Column::Deleted.eq(false))
    }
}
