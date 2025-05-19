use itertools::Itertools;

use crate::{
    db::{datetime_to_str, entities},
    model::data_import::{
        row::{ImportRowUuid, ModelImportRow},
        row_item::{ContentRef, ModelImportRowItem},
        DataImportUuid, ModelDataImport,
    },
};

#[derive(Default)]
pub struct EntitiesToInsert {
    pub imports: Vec<entities::data_import::Model>,
    pub rows: Vec<entities::data_import_row::Model>,
    pub items: Vec<entities::data_import_row_item::Model>,
}

impl EntitiesToInsert {
    pub fn extend(&mut self, import: DestructuredImport) {
        self.imports.push(import.0);
        self.rows.extend(import.1);
        self.items.extend(import.2);
    }
}

impl From<Vec<ModelDataImport>> for EntitiesToInsert {
    fn from(value: Vec<ModelDataImport>) -> Self {
        value.into_iter().map(import_from_model).fold(
            Self::default(),
            |mut acc, dest_import| {
                acc.extend(dest_import);
                acc
            },
        )
    }
}

impl From<DestructuredImport> for EntitiesToInsert {
    fn from(value: DestructuredImport) -> Self {
        Self {
            imports: vec![value.0],
            rows: value.1,
            items: value.2,
        }
    }
}

impl From<ModelDataImport> for EntitiesToInsert {
    fn from(value: ModelDataImport) -> Self {
        import_from_model(value).into()
    }
}

type DestructuredImport = (
    entities::data_import::Model,
    Vec<entities::data_import_row::Model>,
    Vec<entities::data_import_row_item::Model>,
);

fn import_from_model(
    ModelDataImport {
        uuid,
        profile_uuid,
        file_hash,
        file_path,
        datetime_created,
        rows,
    }: ModelDataImport,
) -> DestructuredImport {
    let import = entities::data_import::Model {
        uuid,
        profile: profile_uuid,
        file_hash,
        file_path: file_path.to_string_lossy().into_owned(),
        datetime_created: datetime_to_str(datetime_created),
    };

    let (rows, items) =
        rows.into_iter().map(|row| row_from_model(uuid, row)).fold(
            (vec![], vec![]),
            |(mut row_vec, mut items_vec), (row, items)| {
                row_vec.push(row);
                items_vec.extend(items);
                (row_vec, items_vec)
            },
        );

    (import, rows, items)
}

fn row_from_model(
    origin_import: DataImportUuid,
    ModelImportRow {
        uuid,
        group_uuid,
        items,
        row_content,
        row_index,
    }: ModelImportRow,
) -> (
    entities::data_import_row::Model,
    Vec<entities::data_import_row_item::Model>,
) {
    let items = items
        .into_iter()
        .map(|item| row_item_from_model(uuid, item))
        .collect_vec();

    let row = entities::data_import_row::Model {
        uuid,
        origin_import,
        group_uuid,
        row_content,
        row_index: row_index as i32,
    };

    (row, items)
}

fn row_item_from_model(
    origin_row: ImportRowUuid,
    ModelImportRowItem {
        uuid,
        content,
        parsed_content_ref,
        item_index,
    }: ModelImportRowItem,
) -> entities::data_import_row_item::Model {
    let (datetime_uuid, movement_uuid, text_uuid, special_uuid) =
        match parsed_content_ref {
            ContentRef::Datetime(datetime_uuid) => {
                (Some(datetime_uuid), None, None, None)
            }
            ContentRef::Movement(movement_uuid) => {
                (None, Some(movement_uuid), None, None)
            }
            ContentRef::Text(text) => (None, None, Some(text), None),
            ContentRef::Special(special) => (None, None, None, Some(special)),
            ContentRef::None => (None, None, None, None),
        };
    entities::data_import_row_item::Model {
        uuid,
        origin_row,
        item_index: item_index as i32,
        content,
        datetime_uuid,
        movement_uuid,
        text_uuid,
        special_uuid,
    }
}
