use hermes::{ContainsTables, TablesCollector};
use itertools::Itertools;
use sea_orm::{DatabaseConnection, DbErr, EntityOrSelect, EntityTrait};

use crate::{
    db::{
        combine_types,
        entities::{self, prelude::DataImportRow},
    },
    model::{data_import::{
        row::{ImportRowUuid, ModelImportRow},
        row_item::ModelImportRowItem,
        DataImportUuid,
    }, group::GroupUuid},
};

use super::row_items_query::all_row_items;

pub(super) async fn all_rows(
    db: &DatabaseConnection,
    collector: &mut TablesCollector,
) -> Result<Vec<ImportRowWithOrigin>, DbErr> {
    let rows = DataImportRow::find()
        .select()
        .and_find_tables(collector)
        .all(db)
        .await?
        .into_iter()
        .map(ImportRowWithOrigin::from)
        .collect_vec();

    let items = all_row_items(db, collector).await?;

    Ok(combine_types(
        rows,
        items,
        |o| o.uuid,
        |i| i.row_uuid,
        |row, items| {
            let items = items.into_iter().map(ModelImportRowItem::from);
            row.items.extend(items)
        },
    ))
}

#[derive(Debug, Clone)]
pub struct ImportRowWithOrigin {
    pub origin_import: DataImportUuid,
    pub uuid: ImportRowUuid,
    pub group_uuid: Option<GroupUuid>,
    pub row_content: String,
    pub row_index: i32,
    pub items: Vec<ModelImportRowItem>,
}

impl From<entities::data_import_row::Model> for ImportRowWithOrigin {
    fn from(
        entities::data_import_row::Model {
            uuid,
            origin_import,
            group_uuid,
            row_content,
            row_index,
        }: entities::data_import_row::Model,
    ) -> Self {
        ImportRowWithOrigin {
            uuid,
            group_uuid,
            row_content,
            row_index,
            origin_import,
            items: vec![],
        }
    }
}

impl From<ImportRowWithOrigin> for ModelImportRow {
    fn from(
        ImportRowWithOrigin {
            uuid,
            group_uuid,
            row_content,
            row_index,
            items,
            ..
        }: ImportRowWithOrigin,
    ) -> Self {
        Self {
            uuid,
            group_uuid,
            row_content,
            row_index: row_index as usize,
            items,
        }
    }
}
