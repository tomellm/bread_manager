use std::{fmt::Display, ops::Deref};

use chrono::{DateTime, Local, NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::profiles::error::ProfileError;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ExpenseRecordUuid(pub Uuid);

impl ExpenseRecordUuid {
    fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl From<Uuid> for ExpenseRecordUuid {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl std::ops::Deref for ExpenseRecordUuid {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseRecord {
    datetime_created: DateTime<Local>,
    uuid: ExpenseRecordUuid,
    /// stored as integer as cents
    amount: isize,
    datetime: DateTime<Local>,
    description: Option<DescriptionContainer>,
    data: Vec<ExpenseData>,
    tags: Vec<String>,
    origin: String,
}

impl ExpenseRecord {
    fn new(
        amount: isize,
        datetime: DateTime<Local>,
        data: Vec<ExpenseData>,
        default_tags: Vec<String>,
        origin: String,
    ) -> Self {
        Self {
            datetime_created: Local::now(),
            uuid: ExpenseRecordUuid::new(),
            amount,
            datetime,
            description: None,
            data,
            tags: default_tags,
            origin,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_all(
        datetime_created: DateTime<Local>,
        uuid: Uuid,
        amount: isize,
        datetime: DateTime<Local>,
        description: Option<DescriptionContainer>,
        data: Vec<ExpenseData>,
        tags: Vec<String>,
        origin: String,
    ) -> Self {
        Self {
            datetime_created,
            uuid: ExpenseRecordUuid(uuid),
            amount,
            datetime,
            description,
            data,
            tags,
            origin,
        }
    }
    pub fn created(&self) -> &DateTime<Local> {
        &self.datetime_created
    }
    pub fn uuid(&self) -> &ExpenseRecordUuid {
        &self.uuid
    }
    pub fn amount(&self) -> &isize {
        &self.amount
    }
    pub fn amount_euro(&self) -> f32 {
        self.amount as f32 / 100.
    }
    pub fn amount_euro_f64(&self) -> f64 {
        self.amount as f64 / 100.
    }
    pub fn formatted_amount(&self) -> String {
        format!("{:.2}€", self.amount as f32 / 100.0)
    }
    pub fn datetime(&self) -> &DateTime<Local> {
        &self.datetime
    }
    pub fn description(&self) -> Option<&String> {
        self.description.as_deref()
    }
    pub fn description_container(&self) -> &Option<DescriptionContainer> {
        &self.description
    }
    pub fn data(&self) -> &Vec<ExpenseData> {
        &self.data
    }
    pub fn tags(&self) -> &Vec<String> {
        &self.tags
    }
    pub fn origin(&self) -> &String {
        &self.origin
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExpenseData {
    Expense(usize),
    Income(usize),
    Movement(isize),
    Description(String),
    ExpenseDateTime(DateTime<Local>),
    ExpenseDate(NaiveDate),
    ExpenseTime(NaiveTime),
    Other(String),
}

impl ExpenseData {
    pub fn data_type(&self) -> &str {
        match self {
            Self::Expense(_) => "Expense",
            Self::Income(_) => "Income",
            Self::Movement(_) => "Movement",
            Self::Description(_) => "Description",
            Self::ExpenseDateTime(_) => "ExpenseDateTime",
            Self::ExpenseDate(_) => "ExpenseDate",
            Self::ExpenseTime(_) => "ExpenseTime",
            Self::Other(_) => "Other",
        }
    }

    pub fn exp(val: usize) -> Self {
        Self::Expense(val)
    }
    pub fn inc(val: usize) -> Self {
        Self::Income(val)
    }
    pub fn mov(val: isize) -> Self {
        Self::Movement(val)
    }
    pub fn desc(val: String) -> Self {
        Self::Description(val)
    }
    pub fn datetime(val: DateTime<Local>) -> Self {
        Self::ExpenseDateTime(val)
    }
    pub fn date(val: NaiveDate) -> Self {
        Self::ExpenseDate(val)
    }
    pub fn time(val: NaiveTime) -> Self {
        Self::ExpenseTime(val)
    }
    pub fn other(val: String) -> Self {
        Self::Other(val)
    }
}

impl Display for ExpenseData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Expense(e) => write!(f, "{e}"),
            Self::Income(e) => write!(f, "{e}"),
            Self::Movement(e) => write!(f, "{e}"),
            Self::Description(e) => write!(f, "{e}"),
            Self::ExpenseDateTime(e) => write!(f, "{e}"),
            Self::ExpenseDate(e) => write!(f, "{e}"),
            Self::ExpenseTime(e) => write!(f, "{e}"),
            Self::Other(e) => write!(f, "{e}"),
        }
        
    }
}

#[derive(Debug, Default)]
pub struct ExpenseRecordBuilder {
    amount: Option<isize>,
    datetime: Option<DateTime<Local>>,
    data: Vec<ExpenseData>,
    default_tags: Vec<String>,
    origin: String,
}

impl ExpenseRecordBuilder {
    pub fn amount_split(&mut self, income: usize, expense: usize) {
        self.amount = Some(income as isize - expense as isize);
    }
    pub fn amount_combined(&mut self, movement: isize) {
        self.amount = Some(movement);
    }
    pub fn datetime(&mut self, datetime: DateTime<Local>) {
        self.datetime = Some(datetime);
    }
    pub fn date(&mut self, date: NaiveDate) {
        self.datetime = date
            .and_hms_opt(0, 0, 0)
            .map(|dt| dt.and_local_timezone(Local::now().timezone()).unwrap());
    }
    pub fn date_time(&mut self, date: NaiveDate, time: NaiveTime) {
        self.datetime = Some(date.and_time(time))
            .map(|dt| dt.and_local_timezone(Local::now().timezone()).unwrap());
    }
    pub fn add_data(&mut self, data: ExpenseData) {
        self.data.push(data);
    }
    pub fn default_tags(&mut self, tags: Vec<String>) {
        self.default_tags = tags;
    }
    pub fn origin(&mut self, origin: String) {
        self.origin = origin;
    }
    pub fn build(&self) -> Result<ExpenseRecord, ProfileError> {
        match (self.amount, self.datetime) {
            (Some(amount), Some(datetime)) => Ok(ExpenseRecord::new(
                amount,
                datetime,
                self.data.clone(),
                self.default_tags.clone(),
                self.origin.clone(),
            )),
            _ => Err(ProfileError::build(self.amount, self.datetime)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Description {
    desc: String,
    datetime_created: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescriptionContainer {
    current: Description,
    history: Vec<Description>,
}

impl Description {
    pub fn new(desc: String) -> Self {
        Self {
            desc,
            datetime_created: Local::now(),
        }
    }
}

impl DescriptionContainer {
    pub fn new(desc: String) -> Self {
        Self {
            current: Description::new(desc),
            history: vec![],
        }
    }
}

impl Deref for DescriptionContainer {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.current.desc
    }
}
