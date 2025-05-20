use hermes::{ContainsTables, TablesCollector};
use sea_orm::{
    DatabaseConnection, DbErr, EntityTrait, FromQueryResult, QuerySelect,
};

use crate::{
    db::entities::{self, prelude::*},
    model::transactions::{
        group::GroupUuid,
        movement::{ModelMovement, MovementUuid},
        properties::{OriginType, TransactionProperties, TransactionRelType},
        TransactionUuid,
    },
};

use crate::db::entities::{movement, transaction_movement};

#[derive(FromQueryResult)]
pub(in crate::db) struct MovementOfTransaction {
    pub uuid: MovementUuid,
    pub transaction_uuid: TransactionUuid,
    pub rel_type: TransactionRelType,
    pub origin_type: OriginType,
    pub amount: i32,
    pub group_uuid: GroupUuid,
}

pub(super) async fn all_movements(
    db: &DatabaseConnection,
    collector: &mut TablesCollector,
) -> Result<Vec<MovementOfTransaction>, DbErr> {
    TransactionMovement::find()
        .select_only()
        .column(movement::Column::Uuid)
        .column(transaction_movement::Column::TransactionUuid)
        .column(transaction_movement::Column::RelType)
        .column(movement::Column::OriginType)
        .column(movement::Column::Amount)
        .column(movement::Column::GroupUuid)
        .left_join(Movement)
        .and_find_tables(collector)
        .into_model()
        .all(db)
        .await
}

impl From<MovementOfTransaction> for ModelMovement {
    fn from(value: MovementOfTransaction) -> Self {
        ModelMovement::new(
            value.uuid,
            value.origin_type,
            value.amount,
            value.group_uuid,
        )
    }
}

impl From<MovementOfTransaction> for TransactionProperties {
    fn from(value: MovementOfTransaction) -> Self {
        TransactionProperties::Movement(value.into())
    }
}

pub fn movement_from_model(
    transaction_uuid: TransactionUuid,
    rel_type: TransactionRelType,
    ModelMovement {
        uuid,
        origin_type,
        amount,
        group_uuid,
    }: ModelMovement,
) -> (
    entities::movement::Model,
    entities::transaction_movement::Model,
) {
    let movement = entities::movement::Model {
        uuid,
        origin_type,
        amount,
        group_uuid,
    };
    let link = entities::transaction_movement::Model {
        transaction_uuid,
        movement_uuid: uuid,
        rel_type,
    };
    (movement, link)
}
