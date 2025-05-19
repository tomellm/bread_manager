use std::collections::HashMap;

use chrono::{DateTime, Local};
use itertools::Itertools;
use num_traits::One;
use uuid::Uuid;

use crate::{
    db::{
        entities, parse_datetime_str,
        query::{
            transaction_datetime_query::DatetimeOfTransaction,
            transaction_movement_query::MovementOfTransaction,
        },
    },
    model::{
        self,
        tags::Tag,
        transactions::{
            datetime::ModelDatetime,
            movement::ModelMovement,
            properties::{TransactionProperties, TransactionRelType},
            ModelTransaction, State, TransactionUuid,
        },
    },
};

#[derive(Debug)]
pub struct TransactionBuilder {
    pub uuid: TransactionUuid,
    pub state: model::transactions::State,
    pub datetime: Option<ModelDatetime>,
    pub movement: Option<ModelMovement>,
    pub datetime_created: DateTime<Local>,
    pub properties: Vec<TransactionProperties>,
    pub tags: Vec<Tag>,
}

impl TransactionBuilder {
    pub fn init() -> Self {
        Self {
            uuid: Uuid::new_v4().into(),
            datetime: None,
            movement: None,
            properties: vec![],
            state: State::Active,
            datetime_created: Local::now(),
            tags: vec![],
        }
    }

    pub fn new(model: entities::transaction::Model) -> Self {
        Self {
            uuid: model.uuid,
            datetime: None,
            movement: None,
            properties: vec![],
            state: model.state,
            datetime_created: parse_datetime_str(
                model.datetime_created.as_str(),
            ),
            tags: vec![],
        }
    }

    pub fn feed_datetimes(
        mut self,
        datetimes: &mut HashMap<
            (TransactionUuid, TransactionRelType),
            impl Iterator<Item = DatetimeOfTransaction>,
        >,
    ) -> Self {
        let mut primarys = datetimes
            .remove(&(self.uuid, TransactionRelType::Primary))
            .unwrap()
            .collect_vec();
        assert!(primarys.len().is_one());
        let _ = self.datetime.insert(primarys.remove(0).into());

        self.feed_properties_opt(
            datetimes.remove(&(self.uuid, TransactionRelType::Additional)),
        );

        self
    }

    pub fn feed_movements(
        mut self,
        movements: &mut HashMap<
            (TransactionUuid, TransactionRelType),
            impl Iterator<Item = MovementOfTransaction>,
        >,
    ) -> Self {
        let mut primarys = movements
            .remove(&(self.uuid, TransactionRelType::Primary))
            .unwrap()
            .collect_vec();
        assert!(primarys.len().is_one());
        let _ = self.movement.insert(primarys.remove(0).into());

        self.feed_properties_opt(
            movements.remove(&(self.uuid, TransactionRelType::Additional)),
        );
        self
    }

    pub fn feed_property<T: Into<TransactionProperties>>(
        &mut self,
        property: T,
    ) {
        self.properties.push(property.into());
    }

    pub fn feed_properties_opt<T: Into<TransactionProperties>>(
        &mut self,
        properties: Option<impl Iterator<Item = T>>,
    ) {
        if let Some(properties) = properties {
            self.properties.extend(properties.map(T::into));
        }
    }

    pub fn feed_tags(&mut self, default_tags: impl IntoIterator<Item = Tag>) {
        self.tags.extend(default_tags);
    }

    pub fn build(self) -> ModelTransaction {
        ModelTransaction {
            uuid: self.uuid,
            datetime: self.datetime.unwrap(),
            movement: self.movement.unwrap(),
            properties: self.properties,
            state: self.state,
            datetime_created: self.datetime_created,
        }
    }
}

pub(in crate::db) trait ToTransacHashMap<T> {
    fn to_hashmap(
        self,
    ) -> HashMap<(TransactionUuid, TransactionRelType), impl Iterator<Item = T>>;
}

impl<I, T> ToTransacHashMap<T> for I
where
    I: IntoIterator<Item = T>,
    T: IsTransacRelated + 'static,
{
    fn to_hashmap(
        self,
    ) -> HashMap<(TransactionUuid, TransactionRelType), impl Iterator<Item = T>>
    {
        self.into_iter()
            .chunk_by(IsTransacRelated::key)
            .into_iter()
            .map(|group| (group.0, group.1.collect_vec().into_iter()))
            .collect::<HashMap<_, _>>()
    }
}

pub(in crate::db) trait IsTransacRelated {
    fn key(&self) -> (TransactionUuid, TransactionRelType);
}

impl IsTransacRelated for DatetimeOfTransaction {
    fn key(&self) -> (TransactionUuid, TransactionRelType) {
        (self.transaction_uuid, self.rel_type)
    }
}

impl IsTransacRelated for MovementOfTransaction {
    fn key(&self) -> (TransactionUuid, TransactionRelType) {
        (self.transaction_uuid, self.rel_type)
    }
}
