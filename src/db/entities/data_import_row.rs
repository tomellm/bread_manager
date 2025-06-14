//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.0

use sea_orm::entity::prelude::*;

use crate::model::{
    data_import::{row::ImportRowUuid, DataImportUuid},
    group::GroupUuid,
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "data_import_row")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub uuid: ImportRowUuid,
    pub origin_import: DataImportUuid,
    pub group_uuid: Option<GroupUuid>,
    #[sea_orm(column_type = "Text")]
    pub row_content: String,
    pub row_index: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::data_groups::Entity",
        from = "Column::GroupUuid",
        to = "super::data_groups::Column::Uuid",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    DataGroups,
    #[sea_orm(
        belongs_to = "super::data_import::Entity",
        from = "Column::OriginImport",
        to = "super::data_import::Column::Uuid",
        on_update = "Restrict",
        on_delete = "Cascade"
    )]
    DataImport,
    #[sea_orm(has_many = "super::data_import_row_item::Entity")]
    DataImportRowItem,
}

impl Related<super::data_groups::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DataGroups.def()
    }
}

impl Related<super::data_import::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DataImport.def()
    }
}

impl Related<super::data_import_row_item::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DataImportRowItem.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
