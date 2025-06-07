use crate::{
    db::entities::{self, prelude::*},
    model::{
        profiles::ProfileUuid,
        tags::{ModelTag, Tag, TagUuid},
        transactions::TransactionUuid,
    },
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
    ColumnAsExpr, DatabaseConnection, DbErr, EntityOrSelect, EntityTrait,
    FromQueryResult, IntoActiveModel, QuerySelect, QueryTrait, Related,
    TryGetable,
};
use uuid::Uuid;

use super::super::entities::{profile_tags, tags, transaction_tags};

#[derive(FromQueryResult)]
pub(in crate::db) struct RelatedTag<RelId>
where
    RelId: TryGetable,
{
    pub rel_uuid: RelId,
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
        self.execute(
            Tags::insert(
                entities::tags::Model {
                    uuid,
                    tag,
                    description,
                }
                .into_active_model(),
            )
            .do_nothing(),
        );
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
) -> Result<Vec<RelatedTag<ProfileUuid>>, DbErr> {
    query_related_tags(
        db,
        collector,
        profile_tags::Column::ProfileUuid,
        ProfileTags,
    )
    .await
}

pub(super) async fn all_transaction_tags(
    db: &DatabaseConnection,
    collector: &mut TablesCollector,
) -> Result<Vec<RelatedTag<TransactionUuid>>, DbErr> {
    query_related_tags(
        db,
        collector,
        transaction_tags::Column::TransactionUuid,
        TransactionTags,
    )
    .await
}

async fn query_related_tags<C, R, RelId>(
    db: &DatabaseConnection,
    collector: &mut TablesCollector,
    c: C,
    _: R,
) -> Result<Vec<RelatedTag<RelId>>, DbErr>
where
    C: ColumnAsExpr,
    R: EntityTrait + Related<Tags>,
    RelId: TryGetable,
{
    R::find()
        .select_only()
        .column_as(c, "rel_uuid")
        .column(tags::Column::Uuid)
        .column(tags::Column::Tag)
        .column(tags::Column::Description)
        .right_join(Tags)
        .and_find_tables(collector)
        .into_model()
        .all(db)
        .await
}

pub(super) fn profile_tags_from_models(
    default_tags: Vec<ModelTag>,
    profile_uuid: ProfileUuid,
) -> Vec<entities::profile_tags::Model> {
    default_tags
        .into_iter()
        .map(|t| entities::profile_tags::Model {
            profile_uuid,
            tag_uuid: t.uuid,
        })
        .collect()
}

pub(super) fn transaction_tag_from_models(
    tags: Vec<ModelTag>,
    transaction_uuid: TransactionUuid,
) -> Vec<entities::transaction_tags::Model> {
    tags.into_iter()
        .map(|t| entities::transaction_tags::Model {
            transaction_uuid,
            tag_uuid: t.uuid,
        })
        .collect()
}

impl<RelId> From<RelatedTag<RelId>> for ModelTag
where
    RelId: TryGetable,
{
    fn from(value: RelatedTag<RelId>) -> Self {
        ModelTag::new(value.uuid.into(), value.tag, value.description)
    }
}
