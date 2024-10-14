pub mod money;
pub mod other;
pub mod time;

use money::{Expense, Income, Movement, NumberFormat, PosExpense};
use other::{Description, Other};
use serde::{Deserialize, Serialize};
use time::{ExpenseDate, ExpenseDateTime, ExpenseTime};

use crate::model::records::ExpenseData;

use super::error::ProfileError;

pub trait Parser<T> {
    fn parse_str(&self, str: &str) -> Result<T, ProfileError>;
    fn to_expense_data(&self, str: &str) -> Result<ExpenseData, ProfileError>;
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
            ExpenseColumn::Combined(pos, _) | ExpenseColumn::OnlyExpense(pos, _) => vec![*pos],
        }
    }
    pub fn into_cols(self) -> Vec<(usize, ParsableWrapper)> {
        match self {
            ExpenseColumn::Combined(pos, val) => vec![(pos, ParsableWrapper::Movement(val))],
            ExpenseColumn::Split((pos1, val1), (pos2, val2)) => vec![
                (pos1, ParsableWrapper::Income(val1)),
                (pos2, ParsableWrapper::Expense(val2)),
            ],
            ExpenseColumn::OnlyExpense(pos, val) => vec![(pos, ParsableWrapper::PosExpense(val))],
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
    pub fn date_time(position1: usize, format1: String, position2: usize, format2: String) -> Self {
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
            DateTimeColumn::DateTime(pos, _) | DateTimeColumn::Date(pos, _) => vec![*pos],
            DateTimeColumn::DateAndTime((pos1, _), (pos2, _)) => vec![*pos1, *pos2],
        }
    }
    pub fn into_cols(self) -> Vec<(usize, ParsableWrapper)> {
        match self {
            DateTimeColumn::DateTime(pos, val) => {
                vec![(pos, ParsableWrapper::ExpenseDateTime(val))]
            }
            DateTimeColumn::Date(pos, val) => vec![(pos, ParsableWrapper::ExpenseDate(val))],
            DateTimeColumn::DateAndTime((pos1, val1), (pos2, val2)) => vec![
                (pos1, ParsableWrapper::ExpenseDate(val1)),
                (pos2, ParsableWrapper::ExpenseTime(val2)),
            ],
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParsableWrapper {
    Income(Income),
    Expense(Expense),
    PosExpense(PosExpense),
    Movement(Movement),
    ExpenseDateTime(ExpenseDateTime),
    ExpenseDate(ExpenseDate),
    ExpenseTime(ExpenseTime),
    Description(Description),
    Other(Other),
}

impl ParsableWrapper {
    pub fn to_expense_data(&self, str: &str) -> Result<ExpenseData, ProfileError> {
        match &self {
            ParsableWrapper::Income(e) => e.to_expense_data(str),
            ParsableWrapper::Expense(e) => e.to_expense_data(str),
            ParsableWrapper::PosExpense(e) => e.to_expense_data(str),
            ParsableWrapper::Movement(e) => e.to_expense_data(str),
            ParsableWrapper::ExpenseDateTime(e) => e.to_expense_data(str),
            ParsableWrapper::ExpenseDate(e) => e.to_expense_data(str),
            ParsableWrapper::ExpenseTime(e) => e.to_expense_data(str),
            ParsableWrapper::Description(e) => e.to_expense_data(str),
            ParsableWrapper::Other(e) => e.to_expense_data(str),
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
        Self::Description(Description::default())
    }
    pub fn other() -> Self {
        Self::Other(Other::default())
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
            ParsableWrapper::Other(_) => write!(f, "Other"),
        }
    }
}
