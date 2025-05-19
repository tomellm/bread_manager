use hermes::carrier::execute::ImplExecuteCarrier;

use crate::{
    db::{
        datetime_to_str,
        entities::{self, prelude::*},
        query::{
            transaction_datetime_query::datetime_from_model,
            transaction_movement_query::movement_from_model,
            transaction_query::EntityTrait,
        },
        VecIntoActiveModel,
    },
    model::transactions::{
        datetime::ModelDatetime,
        movement::ModelMovement,
        properties::{TransactionProperties, TransactionRelType},
        ModelTransaction, TransactionUuid,
    },
};

#[derive(Default)]
pub struct TransactionEntityContainer {
    pub transactions: Vec<entities::transaction::Model>,
    pub movements: Vec<entities::movement::Model>,
    pub transaction_movements: Vec<entities::transaction_movement::Model>,
    pub datetimes: Vec<entities::datetime::Model>,
    pub transaction_datetimes: Vec<entities::transaction_datetime::Model>,
}

impl TransactionEntityContainer {
    pub fn insert_everything(self, exec: &mut impl ImplExecuteCarrier) {
        exec.execute_many(|builder| {
            builder
                .execute(Transaction::insert_many(
                    self.transactions.into_active_model_vec(),
                ))
                .execute(Movement::insert_many(
                    self.movements.into_active_model_vec(),
                ))
                .execute(TransactionMovement::insert_many(
                    self.transaction_movements.into_active_model_vec(),
                ))
                .execute(Datetime::insert_many(
                    self.datetimes.into_active_model_vec(),
                ))
                .execute(TransactionDatetime::insert_many(
                    self.transaction_datetimes.into_active_model_vec(),
                ));
        });
    }

    pub(super) fn add_transaction(
        &mut self,
        ModelTransaction {
            uuid,
            datetime,
            movement,
            properties,
            state,
            datetime_created,
        }: ModelTransaction,
    ) {
        self.transactions.push(entities::transaction::Model {
            uuid,
            state,
            datetime_created: datetime_to_str(datetime_created),
        });
        self.add_movement(uuid, TransactionRelType::Primary, movement);
        self.add_datetime(uuid, TransactionRelType::Primary, datetime);
        self.add_properties(uuid, properties);
    }
    fn add_properties(
        &mut self,
        transac_uuid: TransactionUuid,
        props: Vec<TransactionProperties>,
    ) {
        props.into_iter().for_each(|prop| {
            self.add_property(transac_uuid, prop);
        })
    }
    fn add_property(
        &mut self,
        transac_uuid: TransactionUuid,
        property: TransactionProperties,
    ) {
        match property {
            TransactionProperties::Datetime(datetime) => self.add_datetime(
                transac_uuid,
                TransactionRelType::Additional,
                datetime,
            ),
            TransactionProperties::Movement(movement) => self.add_movement(
                transac_uuid,
                TransactionRelType::Additional,
                movement,
            ),
        }
    }
    fn add_movement(
        &mut self,
        transac_uuid: TransactionUuid,
        rel_type: TransactionRelType,
        movement: ModelMovement,
    ) {
        let models = movement_from_model(transac_uuid, rel_type, movement);
        self.movements.push(models.0);
        self.transaction_movements.push(models.1);
    }

    fn add_datetime(
        &mut self,
        transac_uuid: TransactionUuid,
        rel_type: TransactionRelType,
        datetime: ModelDatetime,
    ) {
        let models = datetime_from_model(transac_uuid, rel_type, datetime);
        self.datetimes.push(models.0);
        self.transaction_datetimes.push(models.1);
    }
}
