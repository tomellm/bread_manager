use sea_orm::entity::prelude::*;

use super::{datetime::Datetime, movement::Movement};

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, DeriveActiveEnum, EnumIter, Hash,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(255))")]
pub enum TransactionRelType {
    #[sea_orm(string_value = "Primary")]
    Primary,
    #[sea_orm(string_value = "Additional")]
    Additional,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransactionProperties {
    Datetime(Datetime),
    Movement(Movement),
}

impl From<Datetime> for TransactionProperties {
    fn from(value: Datetime) -> Self {
        Self::Datetime(value)
    }
}

impl From<Movement> for TransactionProperties {
    fn from(value: Movement) -> Self {
        Self::Movement(value)
    }
}

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, DeriveActiveEnum, EnumIter, Hash,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(255))")]
pub enum OriginType {
    #[sea_orm(string_value = "CsvImport")]
    CsvImport,
    #[sea_orm(string_value = "Manual")]
    Manual,
}
