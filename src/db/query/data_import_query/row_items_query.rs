use hermes::{ContainsTables, TablesCollector};
use itertools::Itertools;
use sea_orm::{DatabaseConnection, DbErr, EntityOrSelect, EntityTrait};

use crate::{
    db::entities::{self, prelude::DataImportRowItem},
    model::data_import::{
        row::ImportRowUuid,
        row_item::{ContentRef, ModelImportRowItem, RowItemUuid},
    },
};

#[derive(Debug, Clone)]
pub(super) struct ImportRowItemWithRow {
    pub row_uuid: ImportRowUuid,
    pub uuid: RowItemUuid,
    pub item_index: usize,
    pub content: String,
    pub parsed_content_ref: ContentRef,
}

pub(super) async fn all_row_items(
    db: &DatabaseConnection,
    collector: &mut TablesCollector,
) -> Result<Vec<ImportRowItemWithRow>, DbErr> {
    let items = DataImportRowItem::find()
        .select()
        .and_find_tables(collector)
        .into_model()
        .all(db)
        .await?
        .into_iter()
        .map(to_model_row)
        .collect_vec();

    Ok(items)
}

fn to_model_row(
    value: entities::data_import_row_item::Model,
) -> ImportRowItemWithRow {
    let cont_ref = match (
        value.datetime_uuid,
        value.movement_uuid,
        value.text_uuid,
        value.special_uuid,
    ) {
        (Some(datetime), None, None, None) => ContentRef::Datetime(datetime),
        (None, Some(movement), None, None) => ContentRef::Movement(movement),
        (None, None, Some(text), None) => ContentRef::Text(text),
        (None, None, None, Some(special)) => ContentRef::Special(special),
        (None, None, None, None) => ContentRef::None,
        _ => unreachable!(),
    };
    ImportRowItemWithRow {
        uuid: value.uuid,
        content: value.content,
        parsed_content_ref: cont_ref,
        row_uuid: value.origin_row,
        item_index: value.item_index as usize,
    }
}

impl From<ImportRowItemWithRow> for ModelImportRowItem {
    fn from(
        ImportRowItemWithRow {
            uuid,
            content,
            parsed_content_ref,
            item_index,
            ..
        }: ImportRowItemWithRow,
    ) -> Self {
        Self {
            uuid,
            item_index,
            content,
            parsed_content_ref,
        }
    }
}
