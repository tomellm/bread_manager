use std::{fmt::Display, str::FromStr};

use num_traits::Float;
use serde::{Deserialize, Serialize};

use crate::model::{
    profiles::error::ProfileError,
    transactions::{
        group::GroupUuid, movement::ModelMovement,
        properties::TransactionProperties,
    },
};

use super::{ParsableWrapper, Parser};

#[derive(
    Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Default,
)]
pub enum NumberFormat {
    /// [.] as thousend separator and [,] as comma separator
    #[default]
    European,
    /// [,] as thousend separator and [.] as comma separator
    American,
}

impl NumberFormat {
    pub fn parse<F: FromStr + Float>(
        &self,
        str: &str,
    ) -> Result<F, ProfileError> {
        if str.is_empty() {
            return Ok(F::zero());
        }

        let new_str = match self {
            Self::European => str.replace(".", "").replace(",", "."),
            Self::American => str.replace(",", ""),
        };

        new_str
            .parse::<F>()
            .or(Err(ProfileError::number(str, self)))
    }
}

impl Display for NumberFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Income(pub NumberFormat);

impl From<Income> for ParsableWrapper {
    fn from(value: Income) -> Self {
        ParsableWrapper::Income(value)
    }
}

impl From<&NumberFormat> for Income {
    fn from(value: &NumberFormat) -> Self {
        Self(*value)
    }
}

impl From<NumberFormat> for Income {
    fn from(value: NumberFormat) -> Self {
        Self(value)
    }
}

impl Parser<i32> for Income {
    fn parse_str(&self, str: &str) -> Result<i32, ProfileError> {
        Ok((self.0.parse::<f64>(str)? * 100.0) as i32)
    }

    fn to_property(
        &self,
        group_uuid: GroupUuid,
        str: &str,
    ) -> Result<TransactionProperties, ProfileError> {
        let amount = self.parse_str(str)?;
        Ok(ModelMovement::init(amount, group_uuid).into())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Expense(pub NumberFormat);

impl From<Expense> for ParsableWrapper {
    fn from(value: Expense) -> Self {
        Self::Expense(value)
    }
}

impl From<&NumberFormat> for Expense {
    fn from(value: &NumberFormat) -> Self {
        Self(*value)
    }
}

impl From<NumberFormat> for Expense {
    fn from(value: NumberFormat) -> Self {
        Self(value)
    }
}

impl Parser<i32> for Expense {
    fn parse_str(&self, str: &str) -> Result<i32, ProfileError> {
        Ok((self.0.parse::<f64>(str)? * 100.0) as i32)
    }

    fn to_property(
        &self,
        group_uuid: GroupUuid,
        str: &str,
    ) -> Result<TransactionProperties, ProfileError> {
        let amount = self.parse_str(str)?;
        Ok(ModelMovement::init(amount, group_uuid).into())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PosExpense(pub NumberFormat);

impl From<PosExpense> for ParsableWrapper {
    fn from(value: PosExpense) -> Self {
        Self::PosExpense(value)
    }
}

impl From<&NumberFormat> for PosExpense {
    fn from(value: &NumberFormat) -> Self {
        Self(*value)
    }
}

impl From<NumberFormat> for PosExpense {
    fn from(value: NumberFormat) -> Self {
        Self(value)
    }
}

impl Parser<i32> for PosExpense {
    fn parse_str(&self, str: &str) -> Result<i32, ProfileError> {
        Ok((self.0.parse::<f64>(str)? * -100.0) as i32)
    }

    fn to_property(
        &self,
        group_uuid: GroupUuid,
        str: &str,
    ) -> Result<TransactionProperties, ProfileError> {
        let amount = self.parse_str(str)?;
        Ok(ModelMovement::init(amount, group_uuid).into())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Movement(pub NumberFormat);

impl From<Movement> for ParsableWrapper {
    fn from(value: Movement) -> Self {
        Self::Movement(value)
    }
}

impl From<&NumberFormat> for Movement {
    fn from(value: &NumberFormat) -> Self {
        Self(*value)
    }
}

impl From<NumberFormat> for Movement {
    fn from(value: NumberFormat) -> Self {
        Self(value)
    }
}

impl Parser<i32> for Movement {
    fn parse_str(&self, str: &str) -> Result<i32, ProfileError> {
        Ok((self.0.parse::<f64>(str)? * 100.0) as i32)
    }

    fn to_property(
        &self,
        group_uuid: GroupUuid,
        str: &str,
    ) -> Result<TransactionProperties, ProfileError> {
        let amount = self.parse_str(str)?;
        Ok(ModelMovement::init(amount, group_uuid).into())
    }
}
