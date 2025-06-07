
use hermes::{ContainsTables, TablesCollector};
use sea_orm::{
    DatabaseConnection, DbErr, EntityTrait, FromQueryResult, QuerySelect,
};

use crate::{
    db::{
        entities::{self, prelude::*},
        parse_datetime_str,
    },
    model::{
        profiles::ProfileUuid,
        transactions::content_description::{
            ContentDescriptionUuid, ModelContentDescription,
        },
    },
};

#[derive(FromQueryResult)]
pub(in crate::db) struct ProfileDescription {
    pub profile_uuid: ProfileUuid,
    pub uuid: ContentDescriptionUuid,
    pub description: String,
    pub datetime_created: String,
}

pub(super) async fn all_profile_descriptions(
    db: &DatabaseConnection,
    collector: &mut TablesCollector,
) -> Result<Vec<ProfileDescription>, DbErr> {
    ContentDescription::find()
        .select_only()
        .column_as(entities::profile::Column::Uuid, "profile_uuid")
        .column(entities::content_description::Column::Uuid)
        .column(entities::content_description::Column::Description)
        .column(entities::content_description::Column::DatetimeCreated)
        .left_join(Profile)
        .and_find_tables(collector)
        .into_model::<ProfileDescription>()
        .all(db)
        .await
}

impl From<ProfileDescription> for ModelContentDescription {
    fn from(
        ProfileDescription {
            uuid,
            description,
            datetime_created,
            ..
        }: ProfileDescription,
    ) -> Self {
        Self {
            uuid,
            description,
            datetime_created: parse_datetime_str(&datetime_created),
        }
    }
}
