use crate::model::{
    profiles::error::ProfileError,
    transactions::{
        content_description::ContentDescription,
        group::GroupUuid,
        properties::TransactionProperties,
        special_content::{SpecialContent, SpecialType},
        text_content::TextContent,
    },
};

use super::{ParsableWrapper, Parser};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Description(pub ContentDescription);

impl Description {
    pub fn default_init() -> Self {
        Self(ContentDescription::init(String::default()))
    }
    pub fn init(description: String) -> Self {
        Self(ContentDescription::init(description))
    }
}

impl From<Description> for ParsableWrapper {
    fn from(value: Description) -> Self {
        Self::Description(value)
    }
}

impl Parser<String> for Description {
    fn parse_str(&self, str: &str) -> Result<String, ProfileError> {
        Ok(str.to_owned())
    }

    fn to_property(
        &self,
        group_uuid: GroupUuid,
        str: &str,
    ) -> Result<TransactionProperties, ProfileError> {
        Ok(TransactionProperties::Text(TextContent::init(
            str.to_string(),
            self.0.clone(),
            group_uuid,
        )))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Special(pub SpecialType, pub ContentDescription);

impl Special {
    pub fn default_init() -> Self {
        Self(
            SpecialType::default(),
            ContentDescription::init(String::default()),
        )
    }
    pub fn init(special_type: SpecialType, description: String) -> Self {
        Self(special_type, ContentDescription::init(description))
    }
}

impl From<Special> for ParsableWrapper {
    fn from(value: Special) -> Self {
        Self::Special(value)
    }
}

impl Parser<String> for Special {
    fn parse_str(&self, str: &str) -> Result<String, ProfileError> {
        Ok(str.to_owned())
    }

    fn to_property(
        &self,
        group_uuid: GroupUuid,
        str: &str,
    ) -> Result<TransactionProperties, ProfileError> {
        Ok(TransactionProperties::Special(SpecialContent::init(
            str.to_string(),
            self.1.clone(),
            self.0,
            group_uuid,
        )))
    }
}
