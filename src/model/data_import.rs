pub mod row;
pub mod row_item;

use std::{
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
};

use chrono::{DateTime, Local};
use row::ImportRow;

use crate::{db::InitUuid, uuid_impls};

use super::profiles::ProfileUuid;

pub type ModelDataImport = DataImport;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataImport {
    pub uuid: DataImportUuid,
    pub profile_uuid: ProfileUuid,
    pub file_hash: Vec<u8>,
    pub file_path: PathBuf,
    pub datetime_created: DateTime<Local>,
    pub rows: Vec<ImportRow>,
}

uuid_impls!(DataImportUuid);

impl DataImport {
    pub fn init(
        profile_uuid: ProfileUuid,
        file_contents: &str,
        file_path: PathBuf,
    ) -> Self {
        let mut hasher = DefaultHasher::new();
        file_contents.hash(&mut hasher);
        let hash = hasher.finish();

        Self {
            uuid: DataImportUuid::init(),
            profile_uuid,
            file_hash: hash.to_be_bytes().to_vec(),
            file_path,
            datetime_created: Local::now(),
            rows: vec![],
        }
    }

    pub fn sort_by_index(&mut self) {
        self.rows.sort_by_key(|r| r.row_index);
        self.rows
            .iter_mut()
            .for_each(|r| r.items.sort_by_key(|i| i.item_index));
    }
}
