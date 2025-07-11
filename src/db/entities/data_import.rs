//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.0

use sea_orm::entity::prelude::*;

use crate::model::{data_import::DataImportUuid, profiles::ProfileUuid};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "data_import")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub uuid: DataImportUuid,
    pub profile: ProfileUuid,
    #[sea_orm(column_type = "Blob")]
    pub file_hash: Vec<u8>,
    #[sea_orm(column_type = "Text")]
    pub file_path: String,
    pub datetime_created: String,
    // ToDo - this has to be deletable, but maybe also not because all the
    // recrods depend from this
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::data_import_row::Entity")]
    DataImportRow,
    #[sea_orm(
        belongs_to = "super::profile::Entity",
        from = "Column::Profile",
        to = "super::profile::Column::Uuid",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    Profile,
}

impl Related<super::data_import_row::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DataImportRow.def()
    }
}

impl Related<super::profile::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Profile.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
