use std::{cmp::Ordering, fmt::Display, mem, ops::Deref};

use chrono::{DateTime, Local, NaiveDate, NaiveTime};
use itertools::Itertools;
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

/// stored as integer as cents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseRecord {
    datetime_created: DateTime<Local>,
    uuid: ExpenseRecordUuid,
    amount: isize,
    datetime: DateTime<Local>,
    description: Option<DescriptionContainer>,
    data: Vec<ExpenseData>,
    tags: Vec<String>,
    origin: String,
    data_import: Uuid,
    state: ExpenseRecordState,
}

#[derive(Default, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum ExpenseRecordState {
    #[default]
    Active,
    Ignored,
    Deleted,
}

impl Display for ExpenseRecordState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<String> for ExpenseRecordState {
    fn from(value: String) -> Self {
        // ToDo: instead of writing out the strings I could
        // use a list of the values and compare using to_string
        match value.as_str() {
            "Active" => Self::Active,
            "Ignored" => Self::Ignored,
            "Deleted" => Self::Deleted,
            _ => unreachable!(),
        }
    }
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
        data_import: Uuid,
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
            data_import,
            state: ExpenseRecordState::default(),
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
        data_import: Uuid,
        state: ExpenseRecordState,
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
            data_import,
            state,
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
    pub fn data_import(&self) -> &Uuid {
        &self.data_import
    }
    pub fn state(&self) -> ExpenseRecordState {
        self.state
    }

    pub fn is_same_record(&self, other: &Self) -> bool {
        let desc_overlaps = match (&self.description, &other.description) {
            (Some(this), Some(other)) => this.one_overlaps_with_exact(other),
            _ => true,
        };

        self.amount == other.amount
            && self.origin.eq(&other.origin)
            && desc_overlaps
            && self.datetime.eq(&other.datetime)
    }

    pub fn sorting_fn() -> impl FnMut(&Self, &Self) -> Ordering {
        |a, b| {
            a.datetime()
                .cmp(b.datetime())
                .then(a.amount().cmp(b.amount()))
        }
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
    data_import: Option<Uuid>,
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
    pub fn data_import(&mut self, data_import: Uuid) {
        self.data_import = Some(data_import);
    }
    pub fn build(&self) -> Result<ExpenseRecord, ProfileError> {
        match (self.amount, self.datetime, self.data_import) {
            (Some(amount), Some(datetime), Some(data_import)) => Ok(ExpenseRecord::new(
                amount,
                datetime,
                self.data.clone(),
                self.default_tags.clone(),
                self.origin.clone(),
                self.description.clone().clone(),
                data_import,
            )),
            _ => Err(ProfileError::build(
                self.amount,
                self.datetime,
                self.data_import,
            )),
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

    /// This method checks if this String overlaps with any part of any
    /// field of this object, or any field overlaps with any part of the
    /// passed String
    pub fn str_overlaps_with(&self, other: &str) -> bool {
        let other = other.to_lowercase();
        let this = self.all_str().map(String::as_str).map(str::to_lowercase);

        this.into_iter()
            .any(|this_item| this_item.contains(&other) || other.contains(&this_item))
    }

    /// This method checks if this String overlaps with any part of any
    /// field of this object, or any field overlaps with any part of the
    /// passed String
    pub fn str_overlaps_with_exact(&self, other: &String) -> bool {
        let this = self.all_str();

        this.into_iter().any(|this_item| this_item.eq(other))
    }

    /// This method checks wherether the other object of the same type
    /// has at least one field that exactly overlaps with one field of
    /// this object
    pub fn one_overlaps_with_exact(&self, other: &Self) -> bool {
        let this = self.all_str();
        let others = other.all_str().collect_vec();

        this.into_iter()
            .any(|this_item| others.contains(&this_item))
    }

    fn all_str(&self) -> impl Iterator<Item = &String> {
        let this = self.history.iter().map(|desc| &desc.desc);
        this.chain([&self.current.desc])
    }
}

impl Deref for DescriptionContainer {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.current.desc
    }
}
