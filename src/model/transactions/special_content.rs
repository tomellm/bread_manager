use sea_orm::{DeriveActiveEnum, EnumIter};

use crate::{db::InitUuid, uuid_impls};
use sea_orm::entity::prelude::*;

use super::{content_description::ContentDescription, group::GroupUuid};

pub(crate) type ModelSpecialContent = SpecialContent;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecialContent {
    pub uuid: SpecialContentUuid,
    pub content: String,
    pub description: ContentDescription,
    pub content_type: SpecialType,
    pub group_uuid: GroupUuid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, DeriveActiveEnum, EnumIter)]
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
    Unknown,
}

impl SpecialContent {
    pub fn init(
        content: String,
        description: String,
        content_type: SpecialType,
        group_uuid: GroupUuid,
    ) -> Self {
        let desc = ContentDescription::init(description);
        Self::new(
            SpecialContentUuid::init(),
            content,
            desc,
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
