use chrono::{DateTime, Local};
use hermes::{ContainsTables, TablesCollector};
use sea_orm::{
    DatabaseConnection, DbErr, EntityTrait, FromQueryResult, QuerySelect,
};

use crate::{
    db::{
        datetime_to_str,
        entities::{self, content_description, prelude::*, special_content},
    },
    model::transactions::{
        content_description::{
            ContentDescriptionUuid, ModelContentDescription,
        },
        group::GroupUuid,
        special_content::{
            ModelSpecialContent, SpecialContentUuid, SpecialType,
        },
        TransactionUuid,
    },
};

#[derive(FromQueryResult)]
pub(in crate::db) struct SpecialOfTransaction {
    uuid: SpecialContentUuid,
    content: String,
    special_type: SpecialType,
    group_uuid: GroupUuid,
    description: String,
    description_uuid: ContentDescriptionUuid,
    datetime_created: DateTime<Local>,
}

pub(super) async fn all_datetimes(
    db: &DatabaseConnection,
    collector: &mut TablesCollector,
) -> Result<Vec<SpecialOfTransaction>, DbErr> {
    TextContent::find()
        .select_only()
        .column(special_content::Column::Uuid)
        .column(special_content::Column::Content)
        .column(special_content::Column::SpecialType)
        .column(special_content::Column::GroupUuid)
        .column(special_content::Column::DescriptionUuid)
        .column(content_description::Column::Description)
        .column(content_description::Column::DatetimeCreated)
        .left_join(ContentDescription)
        .and_find_tables(collector)
        .into_model()
        .all(db)
        .await
}

impl From<SpecialOfTransaction> for ModelSpecialContent {
    fn from(
        SpecialOfTransaction {
            uuid,
            content,
            special_type,
            group_uuid,
            description,
            description_uuid,
            datetime_created,
        }: SpecialOfTransaction,
    ) -> Self {
        Self::new(
            uuid,
            content,
            ModelContentDescription::new(
                description_uuid,
                description,
                datetime_created,
            ),
            special_type,
            group_uuid,
        )
    }
}

pub fn special_from_model(
    transaction_uuid: TransactionUuid,
    ModelSpecialContent {
        uuid,
        content,
        description:
            ModelContentDescription {
                uuid: description_uuid,
                description: description_text,
                datetime_created,
            },
        content_type: special_type,
        group_uuid,
    }: ModelSpecialContent,
) -> (
    entities::special_content::Model,
    entities::transaction_special::Model,
    entities::content_description::Model,
) {
    (
        entities::special_content::Model {
            uuid,
            description_uuid,
            content,
            group_uuid,
            special_type,
        },
        entities::transaction_special::Model {
            transaction_uuid,
            special_uuid: uuid,
        },
        entities::content_description::Model {
            uuid: description_uuid,
            description: description_text,
            datetime_created: datetime_to_str(datetime_created),
        },
    )
}
