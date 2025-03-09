use hermes::impl_to_active_model;
use sea_orm::{entity::prelude::*, EntityOrSelect};
use sqlx_projector::projectors::{FromEntity, ToEntity};
use uuid::Uuid;

use crate::model::data_import::DataImport;

#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "data_import")]
pub struct Model {
    #[sea_orm(primary_key)]
    uuid: Uuid,
    imported_at: ChronoDateTimeWithTimeZone,
    profile_used: Uuid,
    deleted: bool,
}

pub(crate) type DbDataImport = Entity;

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::records::Entity")]
    ExpenseRecord,
    #[sea_orm(
        belongs_to = "super::profiles::Entity",
        from = "Column::ProfileUsed",
        to = "super::profiles::Column::Uuid"
    )]
    Profile,
}

impl Related<super::records::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ExpenseRecord.def()
    }
}

impl Related<super::profiles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Profile.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl FromEntity<DataImport> for Model {
    fn from_entity(entity: DataImport) -> Self {
        Self {
            uuid: entity.uuid,
            imported_at: entity.imported_at,
            profile_used: entity.profile_used,
            deleted: entity.deleted,
        }
    }
}

impl ToEntity<DataImport> for Model {
    fn to_entity(self) -> DataImport {
        DataImport {
            uuid: self.uuid,
            imported_at: self.imported_at,
            profile_used: self.profile_used,
            deleted: self.deleted,
        }
    }
}

impl_to_active_model!(DataImport, Model);

impl Entity {
    pub fn find_all_active() -> Select<Self> {
        Self::find().select().filter(Column::Deleted.eq(false))
    }
}
