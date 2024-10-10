mod error;
mod builder;
mod columns;

use error::ProfileError;
use std::collections::{HashMap, HashSet};
use tracing::trace;

use bincode as bc;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

use super::records::{ExpenseData, ExpenseRecord, ExpenseRecordBuilder};

//IDEA: change to this if nessesary https://docs.rs/lexical/latest/lexical/

#[derive(Debug, Clone, PartialEq)]
pub struct Profile {
    pub uuid: Uuid,
    pub name: String,
    pub margins: (usize, usize),
    pub delimiter: char,
    amount: ExpenseColumn,
    datetime: DateTimeColumn,
    other_data: HashMap<usize, ParsableWrapper>,
    pub width: usize,
    pub default_tags: Vec<String>,
    pub origin_name: String,
}

type DbProfileParts = (
    (usize, usize),
    char,
    ExpenseColumn,
    DateTimeColumn,
    HashMap<usize, ParsableWrapper>,
    usize,
    Vec<String>,
);

impl Profile {
    #[allow(clippy::too_many_arguments)]
    fn new(
        name: String,
        amount: ExpenseColumn,
        datetime: DateTimeColumn,
        other_data: Vec<(usize, ParsableWrapper)>,
        margins: (usize, usize),
        delimiter: char,
        mut default_tags: Vec<String>,
        origin_name: String,
    ) -> Self {
        let uuid = Uuid::new_v4();
        let mut positions = other_data.iter().map(|(pos, _)| *pos).collect::<Vec<_>>();
        positions.extend(amount.get_positions());
        positions.extend(datetime.get_positions());
        let profile_width = positions.into_iter().max().unwrap();

        let other_data = other_data.into_iter().collect::<HashMap<_, _>>();

        let tags_set = default_tags.drain(..).collect::<HashSet<_>>();
        default_tags.extend(tags_set);
        Self {
            uuid,
            name,
            margins,
            delimiter,
            amount,
            datetime,
            other_data,
            width: profile_width,
            default_tags,
            origin_name,
        }
    }
    pub fn parse_file(&self, file: &str) -> Result<Vec<ExpenseRecord>, ProfileError> {
        //TODO: dont forget to check that this is correct
        let rows = self.cut_margins(file.lines().collect::<Vec<_>>());
        let res_records = rows
            .into_iter()
            .map(|row| self.parse_row(row))
            .collect::<Vec<_>>();
        trace!(msg = format!("{res_records:?}"));
        if res_records.iter().any(Result::is_err) {
            Err(res_records
                .iter()
                .filter(|e| e.is_err())
                .collect::<Vec<_>>()
                .first()
                .unwrap()
                .as_ref()
                .unwrap_err()
                .clone())
 } else {
            Ok(res_records.into_iter().map(Result::unwrap).collect())
        }
    }

    fn parse_row(&self, row: &str) -> Result<ExpenseRecord, ProfileError> {
        let split_row = self.split_row(row);
        if split_row.len() < self.width {
            return Err(ProfileError::width(self.width, split_row.len()));
        }

        let mut builder = ExpenseRecordBuilder::default();

        let get_from_vec = |pos: usize| -> String { split_row.get(pos).unwrap().to_string() };

        match self.amount {
            ExpenseColumn::Split((pos1, _), (pos2, _)) => {
                builder.amount_split(
                    Income::parse_str(&get_from_vec(pos1))?,
                    Expense::parse_str(&get_from_vec(pos2))?,
                );
            }
            ExpenseColumn::Combined(pos, _) => {
                builder.amount_combined(Movement::parse_str(&get_from_vec(pos))?);
            }
            ExpenseColumn::OnlyExpense(pos, _) => {
                builder.amount_split(0, PosExpense::parse_str(&get_from_vec(pos))?);
            }
        };
        match &self.datetime {
            DateTimeColumn::DateTime(pos, el) => {
                builder.datetime(el.parse_str(&get_from_vec(*pos))?);
            }
            DateTimeColumn::Date(pos, el) => builder.date(el.parse_str(&get_from_vec(*pos))?),
            DateTimeColumn::DateAndTime((pos1, el1), (pos2, el2)) => builder.date_time(
                el1.parse_str(&get_from_vec(*pos1))?,
                el2.parse_str(&get_from_vec(*pos2))?,
            ),
        }

        for (index, element) in split_row.into_iter().enumerate() {
            if let Some(parser) = self.other_data.get(&index) {
                let data = parser.to_expense_data(element)?;
                builder.add_data(data);
            }
        }
        builder.default_tags(self.default_tags.clone());
        builder.origin(self.origin_name.clone());
        builder.build()
    }

    fn split_row(&self, row: &str) -> Vec<String> {
        row.split(self.delimiter).map(str::to_owned).collect()
    }

    fn cut_margins<'a>(&self, mut rows: Vec<&'a str>) -> Vec<&'a str> {
        rows.drain(0..self.margins.0);
        let lines_len = rows.len();
        rows.drain((lines_len - self.margins.1)..lines_len);
        rows
    }

    pub fn to_db(&self) -> (Uuid, String, String, Vec<u8>) {
        (
            self.uuid,
            self.name.clone(),
            self.origin_name.clone(),
            bc::serialize(&(
                self.margins,
                self.delimiter,
                self.amount.clone(),
                self.datetime.clone(),
                self.other_data.clone(),
                self.width,
                self.default_tags.clone(),
            ))
            .unwrap(),
        )
    }

    pub fn from_db(uuid: Uuid, name: String, origin_name: String, data: &[u8]) -> Self {
        let (
            margins, delimiter, amount, datetime, other_data, profile_width, default_tags
        ): DbProfileParts = bc::deserialize(data).unwrap();

        Self {
            uuid,
            name,
            margins,
            delimiter,
            amount,
            datetime,
            other_data,
            width: profile_width,
            default_tags,
            origin_name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpenseColumn {
    Split((usize, Income), (usize, Expense)),
    Combined(usize, Movement),
    OnlyExpense(usize, PosExpense),
}

impl ExpenseColumn {
    pub fn split(income: usize, expense: usize) -> Self {
        Self::Split((income, Income), (expense, Expense))
    }
    pub fn combined(combined: usize) -> Self {
        Self::Combined(combined, Movement)
    }
    pub fn only_expense(expense: usize) -> Self {
        Self::OnlyExpense(expense, PosExpense)
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
    pub fn income() -> Self {
        Self::Income(Income)
    }
    pub fn expense() -> Self {
        Self::Expense(Expense)
    }
    pub fn posexpense() -> Self {
        Self::PosExpense(PosExpense)
    }
    pub fn movement() -> Self {
        Self::Movement(Movement)
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

impl Parsable for ParsableWrapper {
    fn to_expense_data(&self, str: String) -> Result<ExpenseData, ProfileError> {
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
}

impl Parsable for String {
    fn to_expense_data(&self, str: String) -> Result<ExpenseData, ProfileError> {
        Ok(ExpenseData::Description(str.clone()))
    }
}
impl Parsable for Income {
    fn to_expense_data(&self, str: String) -> Result<ExpenseData, ProfileError> {
        Ok(ExpenseData::Income(Income::parse_str(&str)?))
    }
}
impl Parsable for Expense {
    fn to_expense_data(&self, str: String) -> Result<ExpenseData, ProfileError> {
        Ok(ExpenseData::Expense(Expense::parse_str(&str)?))
    }
}
impl Parsable for PosExpense {
    fn to_expense_data(&self, str: String) -> Result<ExpenseData, ProfileError> {
        Ok(ExpenseData::Expense(PosExpense::parse_str(&str)?))
    }
}
impl Parsable for Movement {
    fn to_expense_data(&self, str: String) -> Result<ExpenseData, ProfileError> {
        Ok(ExpenseData::Movement(Movement::parse_str(&str)?))
    }
}
impl Parsable for ExpenseDateTime {
    fn to_expense_data(&self, str: String) -> Result<ExpenseData, ProfileError> {
        Ok(ExpenseData::ExpenseDateTime(self.parse_str(&str)?))
    }
}
impl Parsable for ExpenseTime {
    fn to_expense_data(&self, str: String) -> Result<ExpenseData, ProfileError> {
        Ok(ExpenseData::ExpenseTime(self.parse_str(&str)?))
    }
}
impl Parsable for ExpenseDate {
    fn to_expense_data(&self, str: String) -> Result<ExpenseData, ProfileError> {
        Ok(ExpenseData::ExpenseDate(self.parse_str(&str)?))
    }
}
impl Parsable for Description {
    fn to_expense_data(&self, str: String) -> Result<ExpenseData, ProfileError> {
        Ok(ExpenseData::Description(str))
    }
}
impl Parsable for Other {
    fn to_expense_data(&self, str: String) -> Result<ExpenseData, ProfileError> {
        Ok(ExpenseData::Description(str))
    }
}


