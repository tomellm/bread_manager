use crate::{db::entities::prelude::*, model::tags::ModelTag};
use hermes::{ContainsTables, TablesCollector};
use sea_orm::{
    DatabaseConnection, DbErr, EntityTrait, FromQueryResult, QuerySelect,
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
        .left_join(Profile)
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
        .left_join(Profile)
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
