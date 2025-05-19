pub mod models_to_row_items;
mod row_items_query;
mod row_query;

use std::path::PathBuf;

use hermes::{
    carrier::{
        execute::ImplExecuteCarrier, manual_query::ImplManualQueryCarrier,
        query::ExecutedQuery,
    },
    container::manual,
    ContainsTables, TablesCollector,
};
use itertools::Itertools;
use models_to_row_items::EntitiesToInsert;
use num_traits::Zero;
use row_query::all_rows;
use sea_orm::{
    DatabaseConnection, DbErr, EntityOrSelect, EntityTrait, Insert,
};

use crate::{
    db::{
        combine_types, entities::{self, prelude::*}, parse_datetime_str, VecIntoActiveModel
    },
    model::data_import::{row::ModelImportRow, ModelDataImport},
};

pub trait DataImportQuery {
    fn all(&mut self);

    fn insert(&mut self, import: ModelDataImport);

    fn insert_queries(
        import: Vec<ModelDataImport>,
    ) -> (
        Insert<entities::data_import::ActiveModel>,
        Insert<entities::data_import_row::ActiveModel>,
        Insert<entities::data_import_row_item::ActiveModel>,
    ) {
        let to_insert = EntitiesToInsert::from(import);
        (
            DataImport::insert_many(to_insert.imports.into_active_model_vec()),
            DataImportRow::insert_many(to_insert.rows.into_active_model_vec()),
            DataImportRowItem::insert_many(
                to_insert.items.into_active_model_vec(),
            ),
        )
    }
}

impl DataImportQuery for manual::Container<ModelDataImport> {
    fn all(&mut self) {
        self.manual_query(|db, mut collector| async move {
            let data_imports = all_imports(&db, &mut collector).await;
            ExecutedQuery::new_collector(collector, data_imports)
        });
    }

    fn insert(&mut self, import: ModelDataImport) {
        let (imports, rows, items) = Self::insert_queries(vec![import]);
        self.execute_many(|builder| {
            builder.execute(imports).execute(rows).execute(items);
        });
    }
}

pub(super) async fn all_imports(
    db: &DatabaseConnection,
    collector: &mut TablesCollector,
) -> Result<Vec<ModelDataImport>, DbErr> {
    let imports = DataImport::find()
        .select()
        .and_find_tables(collector)
        .all(db)
        .await?
       .into_iter()
        .map(to_model_import)
        .collect_vec();

    let rows = all_rows(db, collector).await?;

    Ok(combine_types(
        imports,
        rows,
        |o| o.uuid,
        |i| i.origin_import,
        |import, rows| {
            let rows = rows.into_iter().map(ModelImportRow::from);
            assert!(!rows.len().is_zero());
            import.rows.extend(rows);
        },
    ))
}

fn to_model_import(
    entities::data_import::Model {
        uuid,
        profile,
        file_hash,
        file_path,
        datetime_created,
    }: entities::data_import::Model,
) -> ModelDataImport {
    ModelDataImport {
        uuid,
        profile_uuid: profile,
        file_hash,
        file_path: PathBuf::from(file_path),
        datetime_created: parse_datetime_str(&datetime_created),
        rows: vec![],
    }
}
