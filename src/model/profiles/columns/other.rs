use serde::{Deserialize, Serialize};

use crate::model::{
    profiles::error::ProfileError,
    transactions::{group::GroupUuid, properties::TransactionProperties},
};

use super::{ParsableWrapper, Parser};

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Description(pub String);

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
        todo!()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Other(pub String);

impl From<Other> for ParsableWrapper {
    fn from(value: Other) -> Self {
        Self::Other(value)
    }
}

impl Parser<String> for Other {
    fn parse_str(&self, str: &str) -> Result<String, ProfileError> {
        Ok(str.to_owned())
    }

    fn to_property(
        &self,
        group_uuid: GroupUuid,
        str: &str,
    ) -> Result<TransactionProperties, ProfileError> {
        todo!()
    }
}
