use sea_orm::{DeriveActiveEnum, EnumIter};
use serde::{Deserialize, Serialize};

use crate::{db::InitUuid, model::group::GroupUuid, uuid_impls};
use sea_orm::entity::prelude::*;

use super::content_description::ContentDescription;

pub(crate) type ModelSpecialContent = SpecialContent;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecialContent {
    pub uuid: SpecialContentUuid,
    pub content: String,
    pub description: ContentDescription,
    pub content_type: SpecialType,
    pub group_uuid: GroupUuid,
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    DeriveActiveEnum,
    EnumIter,
    Default,
    Serialize,
    Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(255))")]
pub enum SpecialType {
    #[sea_orm(string_value = "CurrencyExchangeRate")]
    CurrencyExchangeRate,
    #[sea_orm(string_value = "OriginalCurrency")]
    OriginalCurrency,
    #[sea_orm(string_value = "ExchangeCommision")]
    ExchangeCommision,
    #[sea_orm(string_value = "TransactionState")]
    TransactionState,
    #[sea_orm(string_value = "TransactionType")]
    TransactionType,
    #[sea_orm(string_value = "AccountBalance")]
    AccountBalance,
    #[sea_orm(string_value = "CompletedDate")]
    CompletedDate,
    #[sea_orm(string_value = "Unknown")]
    #[default]
    Unknown,
}

impl SpecialType {
    pub fn values() -> [Self; 8] {
        [
            Self::CurrencyExchangeRate,
            Self::OriginalCurrency,
            Self::ExchangeCommision,
            Self::TransactionState,
            Self::TransactionType,
            Self::AccountBalance,
            Self::CompletedDate,
            Self::Unknown,
        ]
    }
}

impl SpecialContent {
    pub fn init(
        content: String,
        description: ContentDescription,
        content_type: SpecialType,
        group_uuid: GroupUuid,
    ) -> Self {
        Self::new(
            SpecialContentUuid::init(),
            content,
            description,
            content_type,
            group_uuid,
        )
    }

    pub fn new(
        uuid: SpecialContentUuid,
        content: String,
        description: ContentDescription,
        content_type: SpecialType,
        group_uuid: GroupUuid,
    ) -> Self {
        Self {
            uuid,
            content,
            description,
            content_type,
            group_uuid,
        }
    }
}

uuid_impls!(SpecialContentUuid);
