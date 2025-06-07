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
    DatabaseConnection, DbErr, EntityOrSelect, EntityTrait, QueryTrait,
};

use crate::{
    db::{
        combine_types,
        entities::{self, prelude::*},
        parse_datetime_str, IntoInsertQueries,
    },
    model::data_import::{row::ModelImportRow, ModelDataImport},
};

pub trait DataImportQuery {
    fn all(&mut self);

    fn insert(&mut self, import: ModelDataImport);

    fn insert_queries(
        import: Vec<ModelDataImport>,
    ) -> (
        Vec<impl QueryTrait + Send + 'static>,
        Vec<impl QueryTrait + Send + 'static>,
        Vec<impl QueryTrait + Send + 'static>,
    ) {
        let to_insert = EntitiesToInsert::from(import);
        (
            to_insert.imports.into_insert_queries(|a| {
                DataImport::insert_many(a).do_nothing()
            }),
            to_insert.rows.into_insert_queries(|a| {
                DataImportRow::insert_many(a).do_nothing()
            }),
            to_insert.items.into_insert_queries(|a| {
                DataImportRowItem::insert_many(a).do_nothing()
            }),
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
            builder.execute_many(imports).execute_many(rows).execute_many(items);
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
