pub mod content_description;
pub mod datetime;
pub mod movement;
pub mod properties;
pub mod special_content;
pub mod text_content;

use std::cmp::Ordering;

use chrono::{DateTime, Local};
use datetime::Datetime;
use movement::Movement;
use properties::TransactionProperties;
use sea_orm::entity::prelude::*;

use crate::uuid_impls;

use super::tags::Tag;

pub(crate) type ModelTransaction = Transaction;

#[derive(Debug, Clone)]
pub struct Transaction {
    pub uuid: TransactionUuid,
    pub datetime: Datetime,
    pub movement: Movement,
    pub properties: Vec<TransactionProperties>,
    pub state: State,
    pub datetime_created: DateTime<Local>,
    pub tags: Vec<Tag>,
}

impl Transaction {
    pub fn datetime(&self) -> &DateTime<Local> {
        &self.datetime.datetime
    }

    pub fn amount(&self) -> f64 {
        self.movement.amount as f64 / 100f64
    }

    pub fn sorting_fn() -> impl FnMut(&Self, &Self) -> Ordering {
        |a, b| {
            a.datetime()
                .cmp(b.datetime())
                .then(a.movement.amount.cmp(&b.movement.amount))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, DeriveActiveEnum, EnumIter)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(255))")]
pub enum State {
    #[sea_orm(string_value = "Active")]
    Active,
    #[sea_orm(string_value = "Ignored")]
    Ignored,
    #[sea_orm(string_value = "Deleted")]
    Deleted,
}

uuid_impls!(TransactionUuid);
