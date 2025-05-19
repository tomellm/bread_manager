use std::collections::HashSet;

use crate::{
    db::InitUuid,
    model::{origins::Origin, tags::Tag, transactions::group::GroupUuid},
};

use super::{
    error::ProfileError, DateTimeColumn, ExpenseColumn, ParsableWrapper,
    Profile,
};

// ToDo merge with the other profile builder
#[derive(Debug, Clone, Default)]
pub struct CreateProfileBuilder {
    name: Option<String>,
    col_positions: HashSet<usize>,
    expense_col: Option<ExpenseColumn>,
    datetime_col: Option<DateTimeColumn>,
    other_cols: Vec<(usize, ParsableWrapper)>,
    margins: Option<(usize, usize)>,
    delimiter: Option<char>,
    default_tags: Vec<Tag>,
    origin_name: Option<Origin>,
}

impl CreateProfileBuilder {
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
    pub fn other_cols(
        &mut self,
        vals: Vec<(usize, ParsableWrapper)>,
    ) -> Result<(), ()> {
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
    pub fn default_tags(mut self, default_tags: Vec<Tag>) -> Self {
        self.default_tags = default_tags;
        self
    }
    pub fn origin(&mut self, origin: Origin) {
        self.origin_name = Some(origin);
    }
    pub fn get_from_pos(&self, pos: usize) -> Option<ParsableWrapper> {
        if !self.col_positions.contains(&pos) {
            return None;
        }

        self.other_cols
            .iter()
            .find(|(col_pos, _)| pos.eq(col_pos))
            .map(|(_, wrapper)| wrapper.clone())
            .or_else(|| {
                self.expense_col
                    .clone()
                    .and_then(|v| v.get_from_pos(pos))
                    .or(self
                        .datetime_col
                        .clone()
                        .and_then(|v| v.get_from_pos(pos)))
            })
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
    pub fn from_inter_state(
        state: &IntermediateProfileState,
    ) -> Result<Self, ()> {
        let mut builder = Self::default()
            .name(state.name.clone())
            .margins(state.margin_top, state.margin_btm);

        if let Some(origin) = &state.origin {
            builder.origin(origin.clone());
        }

        if let Some(delimiter) =
            state.delimiter.chars().collect::<Vec<_>>().first()
        {
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
        let group_uuid = GroupUuid::init();

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
                return Err(ProfileError::ColumnWidth(format!(
                    "{pos} is not in bounds"
                )));
            };
            let new_str = el
                .to_property(group_uuid, str.as_str())
                .map(|val| format!("{val:?}"))
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
    pub default_tags: Vec<Tag>,
    pub origin: Option<Origin>,
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
            origin: Some(profile.origin.clone()),
        }
    }
}
