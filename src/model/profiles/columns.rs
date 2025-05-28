pub mod money;
pub mod other;
pub mod time;

use money::{Expense, Income, Movement, NumberFormat, PosExpense};
use other::{Description, Special};
use serde::{Deserialize, Serialize};
use time::{ExpenseDate, ExpenseDateTime, ExpenseTime};

use crate::model::{
    data_import::row_item::ImportRowItem,
    transactions::{
        datetime::{Datetime, ModelDatetime},
        group::GroupUuid,
        movement::ModelMovement,
        properties::TransactionProperties,
    },
};

use super::error::ProfileError;

pub trait Parser<T> {
    fn parse_str(&self, str: &str) -> Result<T, ProfileError>;
    fn to_property(
        &self,
        group_uuid: GroupUuid,
        str: &str,
    ) -> Result<TransactionProperties, ProfileError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpenseColumn {
    Split((usize, Income), (usize, Expense)),
    Combined(usize, Movement),
    OnlyExpense(usize, PosExpense),
}

impl ExpenseColumn {
    pub fn defaul_split() -> Self {
        Self::Split(
            (usize::default(), NumberFormat::default().into()),
            (usize::default(), NumberFormat::default().into()),
        )
    }
    pub fn split(income: usize, expense: usize, format: &NumberFormat) -> Self {
        Self::Split((income, format.into()), (expense, format.into()))
    }
    pub fn default_combined() -> Self {
        Self::Combined(usize::default(), NumberFormat::default().into())
    }
    pub fn combined(combined: usize, format: &NumberFormat) -> Self {
        Self::Combined(combined, format.into())
    }
    pub fn default_only_expense() -> Self {
        Self::OnlyExpense(usize::default(), NumberFormat::default().into())
    }
    pub fn only_expense(expense: usize, format: &NumberFormat) -> Self {
        Self::OnlyExpense(expense, format.into())
    }

    pub fn get_from_pos(self, pos: usize) -> Option<ParsableWrapper> {
        self.into_cols()
            .into_iter()
            .find(|(col_pos, _)| pos.eq(col_pos))
            .map(|(_, wrapper)| wrapper)
    }
    pub fn get_positions(&self) -> Vec<usize> {
        match self {
            ExpenseColumn::Split((pos1, _), (pos2, _)) => vec![*pos1, *pos2],
            ExpenseColumn::Combined(pos, _)
            | ExpenseColumn::OnlyExpense(pos, _) => vec![*pos],
        }
    }
    pub fn into_cols(self) -> Vec<(usize, ParsableWrapper)> {
        match self {
            ExpenseColumn::Combined(pos, val) => {
                vec![(pos, ParsableWrapper::Movement(val))]
            }
            ExpenseColumn::Split((pos1, val1), (pos2, val2)) => vec![
                (pos1, ParsableWrapper::Income(val1)),
                (pos2, ParsableWrapper::Expense(val2)),
            ],
            ExpenseColumn::OnlyExpense(pos, val) => {
                vec![(pos, ParsableWrapper::PosExpense(val))]
            }
        }
    }

    pub fn parse_str(
        &self,
        group_uuid: GroupUuid,
        value_getter: &mut impl FnMut(usize) -> ImportRowItem,
    ) -> Result<(ModelMovement, Vec<ImportRowItem>), ProfileError> {
        match self {
            ExpenseColumn::Split((pos1, income), (pos2, expense)) => {
                let mut item_1 = value_getter(*pos1);
                let mut item_2 = value_getter(*pos2);
                let amount = income.parse_str(&item_1.content)?
                    - expense.parse_str(&item_2.content)?;

                let movement = ModelMovement::init(amount, group_uuid);
                item_1.set_movement_ref(movement.uuid);
                item_2.set_movement_ref(movement.uuid);
                Ok((movement, vec![item_1, item_2]))
            }
            ExpenseColumn::Combined(pos, movement) => {
                let mut item = value_getter(*pos);
                let amount = movement.parse_str(&item.content)?;
                let movement = ModelMovement::init(amount, group_uuid);
                item.set_movement_ref(movement.uuid);
                Ok((movement, vec![item]))
            }
            ExpenseColumn::OnlyExpense(pos, pos_expense) => {
                let mut item = value_getter(*pos);
                let amount = -1i32 * pos_expense.parse_str(&item.content)?;
                let movement = ModelMovement::init(amount, group_uuid);
                item.set_movement_ref(movement.uuid);
                Ok((movement, vec![item]))
            }
        }
    }
}

impl std::fmt::Display for ExpenseColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpenseColumn::Combined(_, _) => write!(f, "Combined"),
            ExpenseColumn::Split(_, _) => write!(f, "Split"),
            ExpenseColumn::OnlyExpense(_, _) => write!(f, "OnlyExpense"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DateTimeColumn {
    DateTime(usize, ExpenseDateTime),
    Date(usize, ExpenseDate),
    DateAndTime((usize, ExpenseDate), (usize, ExpenseTime)),
}

impl DateTimeColumn {
    pub fn date(position: usize, format: String) -> Self {
        Self::Date(position, ExpenseDate(format))
    }
    pub fn new_date() -> Self {
        Self::Date(0, ExpenseDate(String::new()))
    }
    pub fn datetime(position: usize, format: String) -> Self {
        Self::DateTime(position, ExpenseDateTime(format))
    }
    pub fn new_datetime() -> Self {
        Self::DateTime(0, ExpenseDateTime(String::new()))
    }
    pub fn date_time(
        position1: usize,
        format1: String,
        position2: usize,
        format2: String,
    ) -> Self {
        Self::DateAndTime(
            (position1, ExpenseDate(format1)),
            (position2, ExpenseTime(format2)),
        )
    }
    pub fn new_date_time() -> Self {
        Self::DateAndTime(
            (0, ExpenseDate(String::new())),
            (0, ExpenseTime(String::new())),
        )
    }
    pub fn get_from_pos(self, pos: usize) -> Option<ParsableWrapper> {
        self.into_cols()
            .into_iter()
            .find(|(col_pos, _)| pos.eq(col_pos))
            .map(|(_, wrapper)| wrapper)
    }
    pub fn get_positions(&self) -> Vec<usize> {
        match self {
            DateTimeColumn::DateTime(pos, _) | DateTimeColumn::Date(pos, _) => {
                vec![*pos]
            }
            DateTimeColumn::DateAndTime((pos1, _), (pos2, _)) => {
                vec![*pos1, *pos2]
            }
        }
    }
    pub fn into_cols(self) -> Vec<(usize, ParsableWrapper)> {
        match self {
            DateTimeColumn::DateTime(pos, val) => {
                vec![(pos, ParsableWrapper::ExpenseDateTime(val))]
            }
            DateTimeColumn::Date(pos, val) => {
                vec![(pos, ParsableWrapper::ExpenseDate(val))]
            }
            DateTimeColumn::DateAndTime((pos1, val1), (pos2, val2)) => vec![
                (pos1, ParsableWrapper::ExpenseDate(val1)),
                (pos2, ParsableWrapper::ExpenseTime(val2)),
            ],
        }
    }

    pub fn parse_str(
        &self,
        group_uuid: GroupUuid,
        value_getter: &mut impl FnMut(usize) -> ImportRowItem,
    ) -> Result<(ModelDatetime, Vec<ImportRowItem>), ProfileError> {
        match self {
            DateTimeColumn::DateTime(pos, el) => {
                let mut item = value_getter(*pos);
                let datetime = Datetime::init_datetime(
                    el.parse_str(&item.content)?,
                    group_uuid,
                );
                item.set_datetime_ref(datetime.uuid);
                Ok((datetime, vec![item]))
            }
            DateTimeColumn::Date(pos, el) => {
                let mut item = value_getter(*pos);
                let datetime = Datetime::init(
                    el.parse_str(&item.content)?,
                    None,
                    0,
                    group_uuid,
                );
                item.set_datetime_ref(datetime.uuid);
                Ok((datetime, vec![item]))
            }
            DateTimeColumn::DateAndTime((pos_1, el_1), (pos_2, el_2)) => {
                let mut item_1 = value_getter(*pos_1);
                let mut item_2 = value_getter(*pos_2);
                let datetime = Datetime::init(
                    el_1.parse_str(&item_1.content)?,
                    Some(el_2.parse_str(&item_2.content)?),
                    0,
                    group_uuid,
                );
                item_1.set_datetime_ref(datetime.uuid);
                item_2.set_datetime_ref(datetime.uuid);
                Ok((datetime, vec![item_1, item_2]))
            }
        }
    }
}

impl std::fmt::Display for DateTimeColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DateTimeColumn::DateTime(_, _) => write!(f, "DateTime"),
            DateTimeColumn::Date(_, _) => write!(f, "Date"),
            DateTimeColumn::DateAndTime(_, _) => write!(f, "DateAndTime"),
        }
    }
}

pub type ModelParsableWrapper = ParsableWrapper;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsableWrapper {
    Income(Income),
    Expense(Expense),
    PosExpense(PosExpense),
    Movement(Movement),
    ExpenseDateTime(ExpenseDateTime),
    ExpenseDate(ExpenseDate),
    ExpenseTime(ExpenseTime),
    Description(Description),
    Special(Special),
}

impl ParsableWrapper {
    pub fn to_property(
        &self,
        group_uuid: GroupUuid,
        str: &str,
    ) -> Result<TransactionProperties, ProfileError> {
        match self {
            ParsableWrapper::Income(income) => {
                income.to_property(group_uuid, str)
            }
            ParsableWrapper::Expense(expense) => {
                expense.to_property(group_uuid, str)
            }
            ParsableWrapper::PosExpense(pos_expense) => {
                pos_expense.to_property(group_uuid, str)
            }
            ParsableWrapper::Movement(movement) => {
                movement.to_property(group_uuid, str)
            }
            ParsableWrapper::ExpenseDateTime(expense_date_time) => {
                expense_date_time.to_property(group_uuid, str)
            }
            ParsableWrapper::ExpenseDate(expense_date) => {
                expense_date.to_property(group_uuid, str)
            }
            ParsableWrapper::ExpenseTime(expense_time) => {
                expense_time.to_property(group_uuid, str)
            }
            ParsableWrapper::Description(description) => {
                description.to_property(group_uuid, str)
            }
            ParsableWrapper::Special(other) => {
                other.to_property(group_uuid, str)
            }
        }
    }
    pub fn income() -> Self {
        Self::Income(Income::default())
    }
    pub fn expense() -> Self {
        Self::Expense(Expense::default())
    }
    pub fn posexpense() -> Self {
        Self::PosExpense(PosExpense::default())
    }
    pub fn movement() -> Self {
        Self::Movement(Movement::default())
    }
    pub fn expensedatetime() -> Self {
        Self::ExpenseDateTime(ExpenseDateTime::default())
    }
    pub fn expensedate() -> Self {
        Self::ExpenseDate(ExpenseDate::default())
    }
    pub fn expensetime() -> Self {
        Self::ExpenseTime(ExpenseTime::default())
    }
    pub fn description() -> Self {
        Self::Description(Description::default_init())
    }
    pub fn other() -> Self {
        Self::Special(Special::default_init())
    }

    fn is_datetime_type(&self) -> bool {
        matches!(
            self,
            Self::ExpenseDate(_)
                | Self::ExpenseDateTime(_)
                | Self::ExpenseTime(_)
        )
    }

    fn is_money_type(&self) -> bool {
        matches!(
            self,
            Self::Movement(_)
                | Self::Expense(_)
                | Self::PosExpense(_)
                | Self::Income(_)
        )
    }
}

impl std::fmt::Display for ParsableWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ParsableWrapper::Income(_) => write!(f, "Income"),
            ParsableWrapper::Expense(_) => write!(f, "Expense"),
            ParsableWrapper::PosExpense(_) => write!(f, "PosExpense"),
            ParsableWrapper::Movement(_) => write!(f, "Movement"),
            ParsableWrapper::ExpenseDateTime(_) => write!(f, "ExpenseDateTime"),
            ParsableWrapper::ExpenseDate(_) => write!(f, "ExpenseDate"),
            ParsableWrapper::ExpenseTime(_) => write!(f, "ExpenseTime"),
            ParsableWrapper::Description(_) => write!(f, "Description"),
            ParsableWrapper::Special(_) => write!(f, "Other"),
        }
    }
}
