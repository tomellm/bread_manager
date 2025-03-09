use chrono::{DateTime, FixedOffset, Local};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use sqlx_projector::impl_to_database;
use uuid::Uuid;

use crate::db::data_import::DbDataImport;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataImport {
    pub(crate) uuid: Uuid,
    pub(crate) imported_at: DateTime<FixedOffset>,
    pub(crate) profile_used: Uuid,
    pub(crate) deleted: bool,
}

impl DataImport {
    pub fn from_profile_now(profile: Uuid) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            imported_at: Local::now().into(),
            profile_used: profile,
            deleted: false,
        }
    }
}

impl_to_database!(DataImport, <DbDataImport as EntityTrait>::Model);
