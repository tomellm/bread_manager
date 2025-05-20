use crate::{
    db::entities::{self, prelude::*},
    model::tags::{ModelTag, Tag, TagUuid},
};
use hermes::{
    carrier::{
        execute::ImplExecuteCarrier, manual_query::ImplManualQueryCarrier,
        query::ExecutedQuery,
    },
    container::manual,
    ContainsTables, TablesCollector,
};
use sea_orm::{
    DatabaseConnection, DbErr, EntityOrSelect, EntityTrait, FromQueryResult,
    IntoActiveModel, QuerySelect, QueryTrait,
};
use uuid::Uuid;

use super::super::entities::{profile_tags, tags, transaction_tags};

#[derive(FromQueryResult)]
pub(super) struct RelatedTag {
    pub rel_uuid: Uuid,
    pub uuid: Uuid,
    pub tag: String,
    pub description: String,
}

pub trait TagsQuery {
    fn delete_query(to_delete: TagUuid) -> impl QueryTrait + Send + 'static {
        Tags::delete_by_id(to_delete)
    }

    fn delete(&mut self, to_delete: TagUuid);

    fn insert(&mut self, to_insert: ModelTag);

    fn all(&mut self);
}

impl TagsQuery for manual::Container<Tag> {
    fn delete(&mut self, to_delete: TagUuid) {
        self.execute(Self::delete_query(to_delete));
    }

    fn insert(
        &mut self,
        Tag {
            uuid,
            tag,
            description,
        }: ModelTag,
    ) {
        self.execute(Tags::insert(
            entities::tags::Model {
                uuid,
                tag,
                description,
            }
            .into_active_model(),
        ));
    }

    fn all(&mut self) {
        self.manual_query(|db, mut collector| async move {
            let tags = all_tags(&db, &mut collector).await;
            ExecutedQuery::new_collector(collector, tags)
        });
    }
}

pub(super) async fn all_tags(
    db: &DatabaseConnection,
    collector: &mut TablesCollector,
) -> Result<Vec<Tag>, DbErr> {
    Tags::find()
        .select()
        .and_find_tables(collector)
        .into_model()
        .all(db)
        .await
}

pub(super) async fn all_profile_tags(
    db: &DatabaseConnection,
    collector: &mut TablesCollector,
) -> Result<Vec<RelatedTag>, DbErr> {
    Tags::find()
        .select_only()
        .column_as(profile_tags::Column::ProfileUuid, "rel_uuid")
        .column(tags::Column::Uuid)
        .column(tags::Column::Tag)
        .column(tags::Column::Description)
        .right_join(Profile)
        .and_find_tables(collector)
        .into_model()
        .all(db)
        .await
}

pub(super) async fn all_transaction_tags(
    db: &DatabaseConnection,
    collector: &mut TablesCollector,
) -> Result<Vec<RelatedTag>, DbErr> {
    Tags::find()
        .select_only()
        .column_as(transaction_tags::Column::TransactionUuid, "rel_uuid")
        .column(tags::Column::Uuid)
        .column(tags::Column::Tag)
        .column(tags::Column::Description)
        .right_join(Transaction)
        .and_find_tables(collector)
        .into_model()
        .all(db)
        .await
}

impl From<RelatedTag> for ModelTag {
    fn from(value: RelatedTag) -> Self {
        ModelTag::new(value.uuid.into(), value.tag, value.description)
    }
}
