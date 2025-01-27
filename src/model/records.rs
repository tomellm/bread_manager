use std::{fmt::Display, mem, ops::Deref};

use chrono::{DateTime, Local, NaiveDate, NaiveTime};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use sqlx_projector::impl_to_database;
use uuid::Uuid;

use crate::db::records::DbRecord;

use super::profiles::error::ProfileError;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

impl_to_database!(ExpenseRecord, <DbRecord as EntityTrait>::Model);

impl ExpenseRecord {
    fn new(
        amount: isize,
        datetime: DateTime<Local>,
        data: Vec<ExpenseData>,
        default_tags: Vec<String>,
        origin: String,
        description: Option<DescriptionContainer>,
    ) -> Self {
        Self {
            datetime_created: Local::now(),
            uuid: ExpenseRecordUuid::new(),
            amount,
            datetime,
            description,
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
    ExpenseDateTime(DateTime<Local>),
    ExpenseDate(NaiveDate),
    ExpenseTime(NaiveTime),
    Description(String, String),
    Other(String, String),
}

impl ExpenseData {
    pub fn data_type(&self) -> &str {
        match self {
            Self::Expense(_) => "Expense",
            Self::Income(_) => "Income",
            Self::Movement(_) => "Movement",
            Self::Description(..) => "Description",
            Self::ExpenseDateTime(_) => "ExpenseDateTime",
            Self::ExpenseDate(_) => "ExpenseDate",
            Self::ExpenseTime(_) => "ExpenseTime",
            Self::Other(..) => "Other",
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
    pub fn desc(title: String, val: String) -> Self {
        Self::Description(title, val)
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
    pub fn other(title: String, val: String) -> Self {
        Self::Other(title, val)
    }
}

impl Display for ExpenseData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Expense(e) => write!(f, "{e}"),
            Self::Income(e) => write!(f, "{e}"),
            Self::Movement(e) => write!(f, "{e}"),
            Self::Description(t, e) => write!(f, "{t}, {e}"),
            Self::ExpenseDateTime(e) => write!(f, "{e}"),
            Self::ExpenseDate(e) => write!(f, "{e}"),
            Self::ExpenseTime(e) => write!(f, "{e}"),
            Self::Other(t, e) => write!(f, "{t}, {e}"),
        }
    }
}

#[derive(Debug, Default)]
pub struct ExpenseRecordBuilder {
    amount: Option<isize>,
    datetime: Option<DateTime<Local>>,
    data: Vec<ExpenseData>,
    description: Option<DescriptionContainer>,
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
    pub fn push_desc(&mut self, title: String, value: String) {
        if let Some(desc) = &mut self.description {
            desc.push_other(title, value);
        } else {
            self.description = Some(DescriptionContainer::new(title, value));
        }
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
                self.description.clone().clone(),
            )),
            _ => Err(ProfileError::build(self.amount, self.datetime)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Description {
    pub title: String,
    pub desc: String,
    pub datetime_created: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescriptionContainer {
    current: Description,
    history: Vec<Description>,
}

impl Description {
    pub fn new(title: String, desc: String) -> Self {
        Self {
            title,
            desc,
            datetime_created: Local::now(),
        }
    }
}

impl DescriptionContainer {
    pub fn new(title: String, desc: String) -> Self {
        Self {
            current: Description::new(title, desc),
            history: vec![],
        }
    }
    pub fn push_current(&mut self, title: String, desc: String) {
        let old_current = mem::replace(&mut self.current, Description::new(title, desc));
        self.history.push(old_current);
    }
    pub fn push_other(&mut self, title: String, desc: String) {
        self.history.push(Description::new(title, desc));
    }
    pub fn set_current(&mut self, index: usize) {
        let new_current = self.history.remove(index);
        let old_current = mem::replace(&mut self.current, new_current);
        self.history.push(old_current);
    }
    pub fn as_vec(&self) -> Vec<&Description> {
        let mut iter = vec![&self.current];
        iter.extend(self.history.iter());
        iter
    }
}

impl Deref for DescriptionContainer {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.current.desc
    }
}
