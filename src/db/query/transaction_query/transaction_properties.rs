use hermes::carrier::execute::{ImplExecuteCarrier, TransactionBuilder};
use tracing::info;

use crate::{
    db::{
        datetime_to_str,
        entities::{self, prelude::*},
        query::transaction_query::EntityTrait,
        IntoInsertQueries, VecIntoActiveModel,
    },
    model::transactions::{
        datetime::ModelDatetime,
        movement::ModelMovement,
        properties::{TransactionProperties, TransactionRelType},
        special_content::ModelSpecialContent,
        text_content::ModelTextContent,
        ModelTransaction, TransactionUuid,
    },
};

use super::{
    transaction_datetime_query::datetime_from_model,
    transaction_movement_query::movement_from_model,
    transaction_special_query::special_from_model,
    transaction_text_query::text_from_model,
};

#[derive(Default)]
pub struct TransactionEntityContainer {
    pub transactions: Vec<entities::transaction::Model>,

    pub movements: Vec<entities::movement::Model>,
    pub transaction_movements: Vec<entities::transaction_movement::Model>,

    pub datetimes: Vec<entities::datetime::Model>,
    pub transaction_datetimes: Vec<entities::transaction_datetime::Model>,

    pub text_content: Vec<entities::text_content::Model>,
    pub transaction_text: Vec<entities::transaction_text::Model>,

    pub special_content: Vec<entities::special_content::Model>,
    pub transaction_special: Vec<entities::transaction_special::Model>,
}

impl TransactionEntityContainer {
    pub fn add_all_to_transaction<'builder, 'executor>(
        self,
        builder: &'builder mut TransactionBuilder<'executor>,
    ) -> &'builder mut TransactionBuilder<'executor> {
        builder
            .execute_many(self.transactions.into_insert_queries(|a| {
                Transaction::insert_many(a).do_nothing()
            }))
            .execute_many(
                self.movements.into_insert_queries(|a| {
                    Movement::insert_many(a).do_nothing()
                }),
            )
            .execute_many(self.transaction_movements.into_insert_queries(|a| {
                TransactionMovement::insert_many(a).do_nothing()
            }))
            .execute_many(
                self.datetimes.into_insert_queries(|a| {
                    Datetime::insert_many(a).do_nothing()
                }),
            )
            .execute_many(self.transaction_datetimes.into_insert_queries(|a| {
                TransactionDatetime::insert_many(a).do_nothing()
            }))
            .execute_many(self.text_content.into_insert_queries(|a| {
                TextContent::insert_many(a).do_nothing()
            }))
            .execute_many(self.transaction_text.into_insert_queries(|a| {
                TransactionText::insert_many(a).do_nothing()
            }))
            .execute_many(self.special_content.into_insert_queries(|a| {
                SpecialContent::insert_many(a).do_nothing()
            }))
            .execute_many(self.transaction_special.into_insert_queries(|a| {
                TransactionSpecial::insert_many(a).do_nothing()
            }))
    }

    pub fn insert_everything(self, exec: &mut impl ImplExecuteCarrier) {
        exec.execute_many(|builder| {
            self.add_all_to_transaction(builder);
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
            TransactionProperties::Text(text_content) => self.add_text(
                transac_uuid,
                TransactionRelType::Additional,
                text_content,
            ),
            TransactionProperties::Special(special_content) => {
                self.add_special(transac_uuid, special_content)
            }
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

    fn add_text(
        &mut self,
        transac_uuid: TransactionUuid,
        rel_type: TransactionRelType,
        text_content: ModelTextContent,
    ) {
        let models = text_from_model(transac_uuid, rel_type, text_content);
        self.text_content.push(models.0);
        self.transaction_text.push(models.1);
    }

    fn add_special(
        &mut self,
        transac_uuid: TransactionUuid,
        special_content: ModelSpecialContent,
    ) {
        let models = special_from_model(transac_uuid, special_content);
        self.special_content.push(models.0);
        self.transaction_special.push(models.1);
    }
}
