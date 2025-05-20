use hermes::{
    carrier::{
        execute::ImplExecuteCarrier, manual_query::ImplManualQueryCarrier,
        query::ExecutedQuery,
    },
    container::manual,
    ContainsTables, TablesCollector,
};
use sea_orm::{
    DatabaseConnection, DbErr, EntityOrSelect, EntityTrait, IntoActiveModel,
    QueryTrait,
};

use crate::{
    db::entities::{self, prelude::*},
    model::origins::{ModelOrigin, OriginUuid},
};

pub trait OriginsQuery {
    fn insert_query(
        ModelOrigin {
            uuid,
            name,
            description,
        }: ModelOrigin,
    ) -> impl QueryTrait + Send + 'static {
        Origins::insert(
            entities::origins::Model {
                uuid,
                name,
                description,
            }
            .into_active_model(),
        )
    }

    fn insert(&mut self, to_insert: ModelOrigin);

    fn delete_query(to_delete: OriginUuid) -> impl QueryTrait + Send + 'static {
        Origins::delete_by_id(to_delete)
    }

    fn delete(&mut self, to_delete: OriginUuid);

    fn all(&mut self);
}

impl OriginsQuery for manual::Container<ModelOrigin> {
    fn insert(&mut self, to_insert: ModelOrigin) {
        self.execute(Self::insert_query(to_insert));
    }

    fn all(&mut self) {
        self.manual_query(|db, mut collector| async move {
            let origins = all_origins(&db, &mut collector).await;
            ExecutedQuery::new_collector(collector, origins)
        });
    }

    fn delete(&mut self, to_delete: OriginUuid) {
        self.execute(Self::delete_query(to_delete));
    }
}

pub(super) async fn all_origins(
    db: &DatabaseConnection,
    collector: &mut TablesCollector,
) -> Result<Vec<ModelOrigin>, DbErr> {
    Origins::find()
        .select()
        .and_find_tables(collector)
        .into_model()
        .all(db)
        .await
}
