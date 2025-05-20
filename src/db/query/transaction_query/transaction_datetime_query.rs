use hermes::{ContainsTables, TablesCollector};
use sea_orm::{
    DatabaseConnection, DbErr, EntityTrait, FromQueryResult, QuerySelect,
};

use crate::{
    db::{entities::{self, prelude::*}, naive_date_to_str, naive_time_to_str},
    model::transactions::{
        datetime::{DatetimeUuid, ModelDatetime},
        group::GroupUuid,
        properties::{OriginType, TransactionProperties, TransactionRelType},
        TransactionUuid,
    },
};

use crate::db::{
    entities::{datetime, transaction_datetime},
    parse_naive_date_str, parse_naive_time_str,
};

#[derive(FromQueryResult)]
pub(in crate::db) struct DatetimeOfTransaction {
    pub uuid: DatetimeUuid,
    pub transaction_uuid: TransactionUuid,
    pub rel_type: TransactionRelType,
    pub origin_type: OriginType,
    pub date: String,
    pub time: Option<String>,
    pub timezone: i32,
    pub group_uuid: GroupUuid,
}

pub(super) async fn all_datetimes(
    db: &DatabaseConnection,
    collector: &mut TablesCollector,
) -> Result<Vec<DatetimeOfTransaction>, DbErr> {
    TransactionDatetime::find()
        .select_only()
        .column(datetime::Column::Uuid)
        .column(transaction_datetime::Column::TransactionUuid)
        .column(transaction_datetime::Column::RelType)
        .column(datetime::Column::OriginType)
        .column(datetime::Column::Date)
        .column(datetime::Column::Time)
        .column(datetime::Column::Timezone)
        .column(datetime::Column::GroupUuid)
        .left_join(Datetime)
        .and_find_tables(collector)
        .into_model()
        .all(db)
        .await
}

impl From<DatetimeOfTransaction> for ModelDatetime {
    fn from(val: DatetimeOfTransaction) -> Self {
        ModelDatetime::new(
            val.uuid,
            val.origin_type,
            parse_naive_date_str(val.date.as_str()),
            val.time.as_deref().map(parse_naive_time_str),
            val.timezone,
            val.group_uuid,
        )
    }
}

impl From<DatetimeOfTransaction> for TransactionProperties {
    fn from(value: DatetimeOfTransaction) -> Self {
        TransactionProperties::Datetime(value.into())
    }
}

pub fn datetime_from_model(
    transaction_uuid: TransactionUuid,
    rel_type: TransactionRelType,
    ModelDatetime {
        uuid,
        origin_type,
        date,
        time,
        timezone,
        group_uuid,
        ..
    }: ModelDatetime,
) -> (
    entities::datetime::Model,
    entities::transaction_datetime::Model,
) {
    let datetime = entities::datetime::Model {
        uuid,
        origin_type,
        date: naive_date_to_str(date),
        time: time.map(naive_time_to_str),
        timezone,
        group_uuid,
    };
    let link = entities::transaction_datetime::Model {
        transaction_uuid,
        datetime_uuid: uuid,
        rel_type,
    };
    (datetime, link)
}
