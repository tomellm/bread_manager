use sea_orm::entity::prelude::*;

use super::{
    datetime::Datetime, movement::Movement, special_content::SpecialContent,
    text_content::TextContent,
};

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
    Text(TextContent),
    Special(SpecialContent),
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

impl From<TextContent> for TransactionProperties {
    fn from(value: TextContent) -> Self {
        Self::Text(value)
    }
}

impl From<SpecialContent> for TransactionProperties {
    fn from(value: SpecialContent) -> Self {
        Self::Special(value)
    }
}
