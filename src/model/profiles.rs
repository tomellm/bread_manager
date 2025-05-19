pub mod builder;
pub mod columns;
pub mod error;

use chrono::{DateTime, Local};
use columns::{DateTimeColumn, ExpenseColumn, ParsableWrapper};
use error::ProfileError;
use itertools::Itertools;
use sea_orm::{DeriveActiveEnum, EnumIter};
use sea_query::StringLen;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    db::{builders::transaction_builder::TransactionBuilder, InitUuid},
    model::data_import::DataImport,
    uuid_impls,
};

use super::{
    data_import::{row::ImportRow, row_item::ImportRowItem},
    origins::Origin,
    tags::Tag,
    transactions::{group::GroupUuid, Transaction},
};

pub type ModelProfile = Profile;

#[derive(Debug, Clone, PartialEq)]
pub struct Profile {
    pub uuid: ProfileUuid,
    pub name: String,
    pub margins: (usize, usize),
    pub delimiter: char,
    pub(crate) amount: ExpenseColumn,
    pub(crate) datetime: DateTimeColumn,
    pub(crate) other_data: HashMap<usize, ParsableWrapper>,
    pub width: usize,
    pub default_tags: Vec<Tag>,
    pub origin: Origin,
    pub state: State,
    pub datetime_created: DateTime<Local>,
}

uuid_impls!(ProfileUuid);

#[derive(Clone, Copy, Debug, PartialEq, Eq, DeriveActiveEnum, EnumIter)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(255))")]
pub enum State {
    #[sea_orm(string_value = "Active")]
    Active,
    #[sea_orm(string_value = "Deleted")]
    Deleted,
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
        default_tags: Vec<Tag>,
        origin: Origin,
    ) -> Self {
        let width = [
            other_data.iter().map(|(pos, _)| *pos).collect_vec(),
            amount.get_positions(),
            datetime.get_positions(),
        ]
        .concat()
        .into_iter()
        .max()
        .unwrap();

        let other_data = other_data.into_iter().collect::<HashMap<_, _>>();

        Self {
            uuid: Uuid::new_v4().into(),
            name,
            margins,
            delimiter,
            amount,
            datetime,
            other_data,
            width,
            default_tags: default_tags.into_iter().unique().collect_vec(),
            origin,
            state: State::Active,
            datetime_created: Local::now().into(),
        }
    }
    pub fn parse_file(
        &self,
        mut import: DataImport,
    ) -> Result<ParseResult, ProfileError> {
        assert!(import.rows.first().is_some());
        assert!(import.rows.first().unwrap().items.first().is_some());
        assert!(!import
            .rows
            .first()
            .unwrap()
            .items
            .first()
            .unwrap()
            .content
            .is_empty());

        let (transactions, mut profile_errors) =
            import.rows.iter_mut().enumerate().fold(
                (vec![], vec![]),
                |(mut trxs, mut errs), (index, mut row)| {
                    //TODO: dont forget to check that this is correct
                    if index >= self.margins.0 && index < self.margins.1 {
                        match self.parse_row(&mut row) {
                            Ok(trx) => trxs.push(trx),
                            Err(err) => errs.push(err),
                        }
                    }
                    (trxs, errs)
                },
            );

        if !profile_errors.is_empty() {
            Err(profile_errors.remove(0))
        } else {
            Ok(ParseResult::new(transactions, import))
        }
    }

    fn parse_row(
        &self,
        row: &mut ImportRow,
    ) -> Result<Transaction, ProfileError> {
        // ToDo actually creat whole group here
        let group_uuid = GroupUuid::init();

        let mut row_items = row
            .row_content
            .split(self.delimiter)
            .map(str::to_owned)
            .enumerate()
            .map(ImportRowItem::init)
            .collect_vec();
        if row_items.len() < self.width {
            return Err(ProfileError::width(self.width, row_items.len()));
        }

        let mut transac_builder = TransactionBuilder::init();

        {
            let mut get_from_vec =
                |pos: usize| -> ImportRowItem { row_items.remove(pos) };

            let movement =
                self.amount.parse_str(group_uuid, &mut get_from_vec)?;
            let _ = transac_builder.movement.insert(movement.0);

            let datetime =
                self.datetime.parse_str(group_uuid, &mut get_from_vec)?;
            let _ = transac_builder.datetime.insert(datetime.0);

            row.items.extend(movement.1);
            row.items.extend(datetime.1);
        }

        let (props, items) = row_items
            .into_iter()
            .enumerate()
            .filter_map(|(index, item)| {
                self.other_data.get(&index).map(|parser| (item, parser))
            })
            .fold(
                (vec![], vec![]),
                |(mut props, mut items), (mut item, parser)| {
                    let property =
                        parser.to_property(group_uuid, &item.content).unwrap();
                    item.set_property_ref(&property);
                    props.push(property);
                    items.push(item);
                    (props, items)
                },
            );

        transac_builder.properties.extend(props);
        row.items.extend(items);

        transac_builder.feed_tags(self.default_tags.clone());

        row.group_uuid = Some(group_uuid);

        Ok(transac_builder.build())
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
}

pub struct ParseResult {
    pub(crate) rows: Vec<Transaction>,
    pub(crate) import: DataImport,
}

impl ParseResult {
    pub fn new(rows: Vec<Transaction>, import: DataImport) -> Self {
        Self { rows, import }
    }
}
