pub mod builder;
pub mod columns;
pub mod error;

use columns::{DateTimeColumn, ExpenseColumn, ParsableWrapper, Parser};
use error::ProfileError;
use std::collections::{HashMap, HashSet};
use tracing::trace;

use bincode as bc;
use sqlx::types::Uuid;

use super::records::{ExpenseData, ExpenseRecord, ExpenseRecordBuilder};

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

        match &self.amount {
            ExpenseColumn::Split((pos1, income), (pos2, expense)) => {
                builder.amount_split(
                    income.parse_str(&get_from_vec(*pos1))?,
                    expense.parse_str(&get_from_vec(*pos2))?,
                );
            }
            ExpenseColumn::Combined(pos, movement) => {
                builder.amount_combined(movement.parse_str(&get_from_vec(*pos))?);
            }
            ExpenseColumn::OnlyExpense(pos, pos_expense) => {
                builder.amount_split(0, pos_expense.parse_str(&get_from_vec(*pos))?);
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
                let data = parser.to_expense_data(&element)?;
                if let ExpenseData::Description(title, value) = data {
                    builder.push_desc(title, value);
                } else {
                    builder.add_data(data);
                }
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
            margins,
            delimiter,
            amount,
            datetime,
            other_data,
            profile_width,
            default_tags,
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
