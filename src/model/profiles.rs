use eframe::Result;
use std::collections::{HashMap, HashSet};
use tracing::trace;

use bincode as bc;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

use super::records::{ExpenseData, ExpenseRecord, ExpenseRecordBuilder};

//IDEA: change to this if nessesary https://docs.rs/lexical/latest/lexical/
fn to_num(str: &str) -> Result<f64, ProfileError> {
    str.replace('.', "")
        .replace(',', ".")
        .parse::<f64>()
        .or(Err(ProfileError::number(str, "f64")))
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Income;
impl Income {
    pub fn parse_str(str: &str) -> Result<usize, ProfileError> {
        if str.is_empty() {
            return Ok(0);
        }
        Ok((to_num(str)? * 100.0) as usize)
    }
}
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Expense;
impl Expense {
    pub fn parse_str(str: &str) -> Result<usize, ProfileError> {
        if str.is_empty() {
            return Ok(0);
        }
        Ok((to_num(str)? * -100.0) as usize)
    }
}
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PosExpense;
impl PosExpense {
    pub fn parse_str(str: &str) -> Result<usize, ProfileError> {
        if str.is_empty() {
            return Ok(0);
        }
        Ok((to_num(str)? * 100.0) as usize)
    }
}
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Movement;
impl Movement {
    pub fn parse_str(str: &str) -> Result<isize, ProfileError> {
        if str.is_empty() {
            return Ok(0);
        }
        Ok((to_num(str)? * 100.0) as isize)
    }
}
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ExpenseDateTime(pub String);
impl ExpenseDateTime {
    pub fn parse_str(&self, str: &str) -> Result<DateTime<Local>, ProfileError> {
        NaiveDateTime::parse_from_str(str, &self.0)
            .map(|dt| dt.and_local_timezone(Local::now().timezone()).unwrap())
            .or(Err(ProfileError::date(str, &self.0)))
    }
}
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ExpenseDate(pub String);
impl ExpenseDate {
    pub fn parse_str(&self, str: &str) -> Result<NaiveDate, ProfileError> {
        NaiveDate::parse_from_str(str, &self.0).or(Err(ProfileError::date(str, &self.0)))
    }
}
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ExpenseTime(pub String);
impl ExpenseTime {
    pub fn parse_str(&self, str: &str) -> Result<NaiveTime, ProfileError> {
        NaiveTime::parse_from_str(str, &self.0).or(Err(ProfileError::date(str, &self.0)))
    }
}
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Description(pub String);

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Other(pub String);

pub trait Parsable {
    fn to_expense_data(&self, str: String) -> Result<ExpenseData, ProfileError>;
}

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

#[derive(Debug, Clone, Default)]
pub struct ProfileBuilder {
    name: Option<String>,
    col_positions: HashSet<usize>,
    expense_col: Option<ExpenseColumn>,
    datetime_col: Option<DateTimeColumn>,
    other_cols: Vec<(usize, ParsableWrapper)>,
    margins: Option<(usize, usize)>,
    delimiter: Option<char>,
    default_tags: Vec<String>,
    origin_name: Option<String>,
}

impl ProfileBuilder {
    pub fn from_marg_del(top: usize, btm: usize, del: char) -> Self {
        Self {
            margins: Some((top, btm)),
            delimiter: Some(del),
            ..Default::default()
        }
    }
    pub fn name(mut self, val: String) -> Self {
        self.name = Some(val);
        self
    }
    pub fn expense_col(&mut self, val: ExpenseColumn) -> Result<(), ()> {
        self.add_many_pos(val.get_positions())?;
        self.expense_col = Some(val);
        Ok(())
    }
    pub fn datetime_col(&mut self, val: DateTimeColumn) -> Result<(), ()> {
        self.add_many_pos(val.get_positions())?;
        self.datetime_col = Some(val);
        Ok(())
    }
    pub fn other_cols(&mut self, vals: Vec<(usize, ParsableWrapper)>) -> Result<(), ()> {
        self.add_many_pos(vals.iter().map(|t| t.0).collect::<Vec<_>>())?;
        self.other_cols = vals;
        Ok(())
    }
    pub fn margins(mut self, top: usize, btm: usize) -> Self {
        self.margins = Some((top, btm));
        self
    }
    pub fn delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = Some(delimiter);
        self
    }
    pub fn default_tags(mut self, default_tags: Vec<String>) -> Self {
        self.default_tags = default_tags;
        self
    }
    pub fn origin_name(&mut self, origin_name: String) {
        self.origin_name = Some(origin_name);
    }
    pub fn get_from_pos(&self, pos: usize) -> Option<ParsableWrapper> {
        if !self.col_positions.contains(&pos) {
            return None;
        }

        self.other_cols
            .iter()
            .find(|(col_pos, _)| pos.eq(col_pos))
            .map(|(_, wrapper)| wrapper.clone())
            .or(self
                .expense_col
                .clone()
                .and_then(|v| v.get_from_pos(pos))
                .or(self.datetime_col.clone().and_then(|v| v.get_from_pos(pos))))
    }
    pub fn build(self) -> Result<Profile, ()> {
        match (
            self.name,
            self.expense_col,
            self.datetime_col,
            self.margins,
            self.delimiter,
            self.origin_name,
        ) {
            (
                Some(name),
                Some(expense_col),
                Some(datetime_col),
                Some(margins),
                Some(delimiter),
                Some(origin_name),
            ) => Ok(Profile::new(
                name,
                expense_col,
                datetime_col,
                self.other_cols,
                margins,
                delimiter,
                self.default_tags,
                origin_name,
            )),
            _ => Err(()),
        }
    }

    fn add_many_pos(&mut self, pos: Vec<usize>) -> Result<(), ()> {
        if pos.iter().any(|pos| self.col_positions.contains(pos)) {
            return Err(());
        }
        for pos in pos {
            self.col_positions.insert(pos);
        }
        Ok(())
    }
    pub fn from_inter_state(state: &IntermediateProfileState) -> Result<Self, ()> {
        let mut builder = Self::default()
            .name(state.name.clone())
            .margins(state.margin_top, state.margin_btm);

        if !state.origin_name.is_empty() {
            builder.origin_name(state.origin_name.clone());
        }

        if let Some(delimiter) = state.delimiter.chars().collect::<Vec<_>>().first() {
            builder = builder.delimiter(*delimiter);
        }

        if let Some(expense_col) = &state.expense_col {
            builder.expense_col(expense_col.clone())?;
        }
        if let Some(datetime_col) = &state.datetime_col {
            builder.datetime_col(datetime_col.clone())?;
        }

        builder.other_cols(state.other_cols.clone())?;

        Ok(builder.default_tags(state.default_tags.clone()))
    }
    pub fn intermediate_parse(
        &self,
        index: usize,
        row: String,
        total_len: usize,
    ) -> Result<IntermediateParse, ProfileError> {
        if let Some((top, btm)) = self.margins {
            if index < top || index >= (total_len - btm) {
                return Ok(IntermediateParse::None);
            }
        }
        let Some(delimiter) = self.delimiter else {
            return Ok(IntermediateParse::Rows(Ok(row)));
        };
        let mut row = row
            .split(delimiter)
            .map(|val| Ok(val.to_string()))
            .collect::<Vec<_>>();

        let mut other_cols = self.other_cols.clone();
        if let Some(expense_col) = &self.expense_col {
            other_cols.extend(expense_col.clone().into_cols());
        }
        if let Some(datetime_col) = &self.datetime_col {
            other_cols.extend(datetime_col.clone().into_cols());
        }

        for (pos, el) in other_cols {
            let Some(Ok(str)) = row.get(pos) else {
                return Err(ProfileError::ColumnWidth(format!("{pos} is not in bounds")));
            };
            let new_str = el
                .to_expense_data(str.clone())
                .map(|val| val.to_string())
                .map_err(|err| format!("{err:?}"));
            let _ = row.remove(pos);
            row.insert(pos, new_str);
        }

        Ok(IntermediateParse::RowsAndCols(row))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IntermediateParse {
    None,
    Rows(Result<String, String>),
    RowsAndCols(Vec<Result<String, String>>),
}

#[derive(Default, Clone)]
pub struct IntermediateProfileState {
    pub name: String,
    pub margin_top: usize,
    pub margin_btm: usize,
    pub delimiter: String,
    pub expense_col: Option<ExpenseColumn>,
    pub datetime_col: Option<DateTimeColumn>,
    pub other_cols: Vec<(usize, ParsableWrapper)>,
    pub default_tags: Vec<String>,
    pub origin_name: String,
}

impl IntermediateProfileState {
    pub fn from_profile(profile: &Profile) -> Self {
        Self {
            name: profile.name.clone(),
            margin_top: profile.margins.0,
            margin_btm: profile.margins.1,
            delimiter: profile.delimiter.to_string(),
            expense_col: Some(profile.amount.clone()),
            datetime_col: Some(profile.datetime.clone()),
            other_cols: profile
                .other_data
                .iter()
                .map(|(a, b)| (*a, b.clone()))
                .collect(),
            default_tags: profile.default_tags.clone(),
            origin_name: profile.origin_name.clone(),
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

#[derive(Debug, Clone)]
pub enum ProfileError {
    NumberParsing(String),
    DateParsing(String),
    ColumnWidth(String),
    BuildRecord(String),
}

impl ProfileError {
    pub fn number(str: &str, n_type: &str) -> Self {
        Self::NumberParsing(format!(
            "Parsing this string: {str} to this type: {n_type} failed"
        ))
    }
    pub fn date(str: &str, format: &str) -> Self {
        Self::DateParsing(format!(
            "This format: {format} does not fit this date string: {str}"
        ))
    }
    pub fn width(expected: usize, actual: usize) -> Self {
        Self::ColumnWidth(format!(
            "The profile expects a minimum width of {expected} but got a width of {actual}"
        ))
    }
    pub fn build(amount: Option<isize>, date: Option<DateTime<Local>>) -> Self {
        Self::BuildRecord(format!(
            "One of these two is not present: {amount:?} {date:?}"
        ))
    }
}
