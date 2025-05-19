use std::collections::HashMap;

use chrono::{DateTime, Local};
use hermes::{
    carrier::{execute::ImplExecuteCarrier, manual_query::ImplManualQueryCarrier, query::ExecutedQuery},
    container::manual,
    ContainsTables, TablesCollector,
};
use itertools::Itertools;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, FromQueryResult,
    IntoActiveModel, QueryFilter, QuerySelect, QueryTrait,
};

use crate::{
    db::{
        combine_types, datetime_to_str,
        entities::{datetime, prelude::*},
    },
    model::{
        origins::{ModelOrigin, OriginUuid},
        profiles::{
            columns::{DateTimeColumn, ExpenseColumn, ParsableWrapper},
            ModelProfile, ProfileUuid, State,
        },
        tags::ModelTag,
    },
};

use super::{
    super::{entities, parse_datetime_str},
    tags_query::all_profile_tags,
};

pub trait ProfileQuery {
    fn all(&mut self);

    fn deleted_query(
        to_delete: &ProfileUuid,
    ) -> impl QueryTrait + Send + 'static {
        Profile::update_many()
            .filter(entities::profile::Column::Uuid.eq(**to_delete))
            .col_expr(entities::profile::Column::State, State::Deleted.into())
    }

    fn deleted_many_query(
        to_delete: impl IntoIterator<Item = ProfileUuid>,
    ) -> impl QueryTrait + Send + 'static {
        Profile::update_many()
            .filter(entities::profile::Column::Uuid.is_in(to_delete))
            .col_expr(entities::profile::Column::State, State::Deleted.into())
    }

    fn insert_query(
        to_insert: ModelProfile,
    ) -> impl QueryTrait + Send + 'static {
        Profile::insert(profile_from_model(to_insert).into_active_model())
    }

    fn insert(&mut self, to_insert: ModelProfile);
}

pub(super) async fn all_profiles(
    db: &DatabaseConnection,
    collector: &mut TablesCollector,
) -> Result<Vec<ModelProfile>, DbErr> {
    let builders = Profile::find()
        .select_only()
        .columns([
            entities::profile::Column::Uuid,
            entities::profile::Column::Name,
            entities::profile::Column::TopMargin,
            entities::profile::Column::BottomMargin,
            entities::profile::Column::Delimiter,
            entities::profile::Column::Amount,
            entities::profile::Column::Datetime,
            entities::profile::Column::OtherData,
            entities::profile::Column::Width,
            entities::profile::Column::State,
            entities::profile::Column::DatetimeCreated,
            entities::profile::Column::OriginUuid,
        ])
        .column_as(entities::origins::Column::Name, "origin_name")
        .column_as(entities::origins::Column::Description, "origin_description")
        .left_join(Origins)
        .and_find_tables(collector)
        .into_model::<ProfileWithOrigin>()
        .all(db)
        .await?
        .into_iter()
        .map(ProfileBuilder::new)
        .collect_vec();

    let profile_tags = all_profile_tags(db, collector).await?;

    let builders = combine_types(
        builders,
        profile_tags,
        |p| p.uuid,
        |t| ProfileUuid::from(t.rel_uuid),
        |builder, tags| {
            let tags = tags.into_iter().map(ModelTag::from);
            builder.default_tags.extend(tags);
        },
    );

    Ok(builders
        .into_iter()
        .map(ProfileBuilder::build)
        .collect_vec())
}

impl ProfileQuery for manual::Container<ModelProfile> {
    fn all(&mut self) {
        self.manual_query(|db, mut collector| async move {
            let profiles = all_profiles(&db, &mut collector).await;
            ExecutedQuery::new_collector(collector, profiles)
        });
    }

    fn insert(&mut self, to_insert: ModelProfile) {
        let query = Self::insert_query(to_insert);
        // ToDo, origins and tags also need to be inserted here
        self.execute(query);
    }
}

pub struct ProfileBuilder {
    uuid: ProfileUuid,
    name: String,
    margins: (usize, usize),
    delimiter: char,
    amount: ExpenseColumn,
    datetime: DateTimeColumn,
    other_data: HashMap<usize, ParsableWrapper>,
    width: usize,
    default_tags: Vec<ModelTag>,
    origin: ModelOrigin,
    state: State,
    datetime_created: DateTime<Local>,
}

impl ProfileBuilder {
    pub fn new(profile: ProfileWithOrigin) -> Self {
        let origin = ModelOrigin::new(
            profile.origin_uuid,
            profile.origin_name,
            profile.origin_description,
        );

        Self {
            uuid: profile.uuid,
            name: profile.name,
            margins: (
                profile.top_margin.try_into().unwrap(),
                profile.bottom_margin.try_into().unwrap(),
            ),
            delimiter: profile.delimiter.chars().nth(0).unwrap(),
            amount: serde_json::from_str(&profile.amount).unwrap(),
            datetime: serde_json::from_str(&profile.datetime).unwrap(),
            other_data: serde_json::from_str(&profile.other_data).unwrap(),
            width: profile.width.try_into().unwrap(),
            default_tags: vec![],
            origin,
            state: profile.state,
            datetime_created: parse_datetime_str(&profile.datetime_created),
        }
    }

    pub fn build(self) -> ModelProfile {
        ModelProfile {
            uuid: self.uuid,
            name: self.name,
            margins: self.margins,
            delimiter: self.delimiter,
            amount: self.amount,
            datetime: self.datetime,
            other_data: self.other_data,
            width: self.width,
            default_tags: self.default_tags,
            origin: self.origin,
            state: self.state,
            datetime_created: self.datetime_created,
        }
    }
}

#[derive(FromQueryResult)]
struct ProfileWithOrigin {
    pub uuid: ProfileUuid,
    pub name: String,
    pub top_margin: i32,
    pub bottom_margin: i32,
    pub delimiter: String,
    pub amount: String,
    pub datetime: String,
    pub other_data: String,
    pub width: i32,
    pub state: State,
    pub datetime_created: String,
    pub origin_uuid: OriginUuid,
    pub origin_name: String,
    pub origin_description: String,
}

fn profile_from_model(
    ModelProfile {
        uuid,
        name,
        margins,
        delimiter,
        amount,
        datetime,
        other_data,
        width,
        default_tags: _default_tags,
        origin,
        state,
        datetime_created,
    }: ModelProfile,
) -> entities::profile::Model {
    entities::profile::Model {
        uuid,
        name,
        top_margin: margins.0 as i32,
        bottom_margin: margins.1 as i32,
        delimiter: delimiter.into(),
        amount: serde_json::ser::to_string(&amount).unwrap(),
        datetime: serde_json::ser::to_string(&datetime).unwrap(),
        other_data: serde_json::ser::to_string(&other_data).unwrap(),
        width: width as i32,
        origin_uuid: origin.uuid,
        state,
        datetime_created: datetime_to_str(datetime_created),
    }
}
