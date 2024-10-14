use serde::{Deserialize, Serialize};

use crate::model::{profiles::error::ProfileError, records::ExpenseData};

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
    fn to_expense_data(&self, str: &str) -> Result<ExpenseData, ProfileError> {
        Ok(ExpenseData::Description(self.0.clone(), self.parse_str(str).unwrap()))
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
    fn to_expense_data(&self, str: &str) -> Result<ExpenseData, ProfileError> {
        Ok(ExpenseData::Other(self.0.clone(), self.parse_str(str).unwrap()))
    }
}


