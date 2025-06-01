use chrono::{DateTime, Local};
use hermes::{ContainsTables, TablesCollector};
use sea_orm::{
    DatabaseConnection, DbErr, EntityTrait, FromQueryResult, QuerySelect,
};

use crate::{
    db::{
        datetime_to_str,
        entities::{self, content_description, prelude::*, text_content},
    },
    model::{
        group::GroupUuid,
        transactions::{
            content_description::{
                ContentDescriptionUuid, ModelContentDescription,
            },
            properties::TransactionRelType,
            text_content::{ModelTextContent, TextContentUuid},
            TransactionUuid,
        },
    },
};

#[derive(FromQueryResult)]
pub(in crate::db) struct TextOfTransaction {
    uuid: TextContentUuid,
    content: String,
    group_uuid: GroupUuid,
    description: String,
    description_uuid: ContentDescriptionUuid,
    datetime_created: DateTime<Local>,
}

pub(super) async fn all_datetimes(
    db: &DatabaseConnection,
    collector: &mut TablesCollector,
) -> Result<Vec<TextOfTransaction>, DbErr> {
    TextContent::find()
        .select_only()
        .column(text_content::Column::Uuid)
        .column(text_content::Column::Content)
        .column(text_content::Column::GroupUuid)
        .column(text_content::Column::DescriptionUuid)
        .column(content_description::Column::Description)
        .column(content_description::Column::DatetimeCreated)
        .left_join(ContentDescription)
        .and_find_tables(collector)
        .into_model()
        .all(db)
        .await
}

impl From<TextOfTransaction> for ModelTextContent {
    fn from(
        TextOfTransaction {
            uuid,
            content,
            group_uuid,
            description,
            description_uuid,
            datetime_created,
        }: TextOfTransaction,
    ) -> Self {
        Self::new(
            uuid,
            content,
            ModelContentDescription::new(
                description_uuid,
                description,
                datetime_created,
            ),
            group_uuid,
        )
    }
}

pub fn text_from_model(
    transaction_uuid: TransactionUuid,
    rel_type: TransactionRelType,
    ModelTextContent {
        uuid,
        content,
        description:
            ModelContentDescription {
                uuid: description_uuid,
                ..
            },
        group_uuid,
    }: ModelTextContent,
) -> (
    entities::text_content::Model,
    entities::transaction_text::Model,
) {
    (
        entities::text_content::Model {
            uuid,
            description_uuid,
            content,
            group_uuid,
        },
        entities::transaction_text::Model {
            transaction_uuid,
            text_uuid: uuid,
            rel_type,
        },
    )
}
