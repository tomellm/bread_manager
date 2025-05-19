use uuid::Uuid;

use crate::{db::InitUuid, model::transactions::group::GroupUuid, uuid_impls};

use super::row_item::ImportRowItem;

pub type ModelImportRow = ImportRow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportRow {
    pub uuid: ImportRowUuid,
    pub group_uuid: Option<GroupUuid>,
    pub row_content: String,
    pub row_index: usize,
    pub items: Vec<ImportRowItem>,
    // ToDo - add group uuid?
}

impl ImportRow {
    pub fn init(row_content: String, row_index: usize) -> Self {
        Self::new(ImportRowUuid::init(), None, row_content, row_index, vec![])
    }
    pub fn new(
        uuid: ImportRowUuid,
        group_uuid: Option<GroupUuid>,
        row_content: String,
        row_index: usize,
        items: Vec<ImportRowItem>,
    ) -> Self {
        Self {
            uuid,
            group_uuid,
            row_content,
            row_index,
            items,
        }
    }
}

uuid_impls!(ImportRowUuid);
