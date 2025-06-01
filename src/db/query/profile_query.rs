use std::collections::HashMap;

use chrono::{DateTime, Local};
use hermes::{
    carrier::{
        execute::ImplExecuteCarrier, manual_query::ImplManualQueryCarrier,
        query::ExecutedQuery,
    },
    container::manual,
    ContainsTables, TablesCollector,
};
use itertools::Itertools;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, FromQueryResult,
    IntoActiveModel, QueryFilter, QuerySelect, QueryTrait, Select,
};

use crate::{
    db::{
        combine_types, datetime_to_str,
        entities::{prelude::*, profile::ParsableWrapper},
        VecIntoActiveModel,
    },
    model::{
        origins::{ModelOrigin, OriginUuid},
        profiles::{
            columns::{
                self, DateTimeColumn, ExpenseColumn, ModelParsableWrapper,
            },
            ModelProfile, ProfileUuid, State,
        },
        tags::ModelTag,
        transactions::content_description::ModelContentDescription,
    },
};

use super::{
    super::{entities, parse_datetime_str},
    content_description_query::{all_profile_descriptions, ProfileDescription},
    tags_query::{all_profile_tags, profile_tags_from_models},
};

pub trait ProfileQuery {
    fn all(&mut self);

    fn all_active(&mut self);

    fn deleted_query(
        to_delete: &ProfileUuid,
    ) -> impl QueryTrait + Send + 'static {
        Profile::update_many()
            .filter(entities::profile::Column::Uuid.eq(**to_delete))
            .col_expr(entities::profile::Column::State, State::Deleted.into())
    }

    fn delete(&mut self, to_delete: &ProfileUuid);

    fn deleted_many_query(
        to_delete: impl IntoIterator<Item = ProfileUuid>,
    ) -> impl QueryTrait + Send + 'static {
        Profile::update_many()
            .filter(entities::profile::Column::Uuid.is_in(to_delete))
            .col_expr(entities::profile::Column::State, State::Deleted.into())
    }

    fn insert_query(
        to_insert: ModelProfile,
    ) -> (
        impl QueryTrait + Send + 'static,
        impl QueryTrait + Send + 'static,
        impl QueryTrait + Send + 'static,
        impl QueryTrait + Send + 'static,
    ) {
        let (profile, descs, profile_descs, tags) =
            profile_from_model(to_insert);
        (
            Profile::insert(profile.into_active_model()).do_nothing(),
            ContentDescription::insert_many(descs.into_active_model_vec())
                .do_nothing(),
            ProfileContentDescriptions::insert_many(
                profile_descs.into_active_model_vec(),
            )
            .do_nothing(),
            ProfileTags::insert_many(tags.into_active_model_vec()).do_nothing(),
        )
    }

    fn insert(&mut self, to_insert: ModelProfile);
}

pub(super) async fn all_profiles(
    db: &DatabaseConnection,
    collector: &mut TablesCollector,
) -> Result<Vec<ModelProfile>, DbErr> {
    all_profiles_core(db, collector, None).await
}

pub(super) async fn all_profiles_filtered(
    db: &DatabaseConnection,
    collector: &mut TablesCollector,
    filters: impl Fn(Select<Profile>) -> Select<Profile> + Send + 'static,
) -> Result<Vec<ModelProfile>, DbErr> {
    all_profiles_core(db, collector, Some(Box::new(filters))).await
}

async fn all_profiles_core(
    db: &DatabaseConnection,
    collector: &mut TablesCollector,
    filters: Option<
        Box<dyn Fn(Select<Profile>) -> Select<Profile> + Send + 'static>,
    >,
) -> Result<Vec<ModelProfile>, DbErr> {
    let base_query = Profile::find()
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
        .left_join(Origins);

    let filtered_query = match filters {
        Some(filters) => filters(base_query),
        None => base_query,
    };

    let builders = filtered_query
        .and_find_tables(collector)
        .into_model::<ProfileWithOrigin>()
        .all(db)
        .await?
        .into_iter()
        .map(ProfileBuilder::new)
        .collect_vec();

    let descs = all_profile_descriptions(db, collector).await?;

    let builders = combine_types(
        builders,
        descs,
        |p| p.uuid,
        |d| d.profile_uuid,
        |builder, desc| {
            builder.desc_containers(
                desc.into_iter().map(ProfileDescription::into).collect(),
            );
        },
    );

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

    fn all_active(&mut self) {
        self.manual_query(|db, mut collector| async move {
            let profiles = all_profiles_filtered(
                &db,
                &mut collector,
                |query: Select<Profile>| {
                    query.filter(
                        entities::profile::Column::State.eq(State::Active),
                    )
                },
            )
            .await;
            ExecutedQuery::new_collector(collector, profiles)
        });
    }

    fn insert(&mut self, to_insert: ModelProfile) {
        let (profs, descs, prof_descs, tags) = Self::insert_query(to_insert);
        // ToDo, origins and tags also need to be inserted here
        self.execute_many(|transac| {
            transac
                .execute(profs)
                .execute(descs)
                .execute(prof_descs)
                .execute(tags);
        });
    }

    fn delete(&mut self, to_delete: &ProfileUuid) {
        self.execute(Self::deleted_query(to_delete));
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
    desc_containers: Option<Vec<ModelContentDescription>>,
}

impl ProfileBuilder {
    pub fn new(profile: ProfileWithOrigin) -> Self {
        let origin = ModelOrigin::new(
            profile.origin_uuid,
            profile.origin_name,
            profile.origin_description,
        );

        let other_data =
            serde_json::from_str::<HashMap<usize, ParsableWrapper>>(
                &profile.other_data,
            )
            .unwrap();

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
            other_data,
            width: profile.width.try_into().unwrap(),
            default_tags: vec![],
            origin,
            state: profile.state,
            datetime_created: parse_datetime_str(&profile.datetime_created),
            desc_containers: None,
        }
    }

    pub fn desc_containers(
        &mut self,
        desc_containers: Vec<ModelContentDescription>,
    ) {
        self.desc_containers.insert(desc_containers);
    }

    pub fn build(self) -> ModelProfile {
        ModelProfile {
            uuid: self.uuid,
            name: self.name,
            margins: self.margins,
            delimiter: self.delimiter,
            amount: self.amount,
            datetime: self.datetime,
            other_data: parsable_wrappers_to_model(
                self.other_data,
                self.desc_containers.unwrap(),
            ),
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
        default_tags,
        origin,
        state,
        datetime_created,
    }: ModelProfile,
) -> (
    entities::profile::Model,
    Vec<entities::content_description::Model>,
    Vec<entities::profile_content_descriptions::Model>,
    Vec<entities::profile_tags::Model>,
) {
    // ToDo: move serialization from this function to
    // here and serialize the whole hashmap instead of
    // just the single wrapper
    let (other_data, descs, profile_descs) =
        parsable_wrappers_from_model(uuid, other_data);
    let profile_tags = profile_tags_from_models(default_tags, &uuid);

    (
        entities::profile::Model {
            uuid,
            name,
            top_margin: margins.0 as i32,
            bottom_margin: margins.1 as i32,
            delimiter: delimiter.into(),
            amount: serde_json::ser::to_string(&amount).unwrap(),
            datetime: serde_json::ser::to_string(&datetime).unwrap(),
            other_data,
            width: width as i32,
            origin_uuid: origin.uuid,
            state,
            datetime_created: datetime_to_str(datetime_created),
        },
        descs,
        profile_descs,
        profile_tags,
    )
}

fn parsable_wrappers_to_model(
    map: HashMap<usize, ParsableWrapper>,
    descs: Vec<ModelContentDescription>,
) -> HashMap<usize, ModelParsableWrapper> {
    let mut descs = descs
        .into_iter()
        .into_group_map_by(|desc| desc.uuid)
        .into_iter()
        .map(|(pos, mut w)| {
            assert!(w.len() == 1);
            (pos, w.remove(0))
        })
        .collect::<HashMap<_, _>>();

    map.into_iter()
        .map(|(pos, wrap)| {
            let desc = wrap.need_desc().map(|uuid| descs.remove(uuid).unwrap());
            (pos, (wrap, desc).into())
        })
        .collect()
}

fn parsable_wrappers_from_model(
    profile_uuid: ProfileUuid,
    model: HashMap<usize, ModelParsableWrapper>,
) -> (
    String,
    Vec<entities::content_description::Model>,
    Vec<entities::profile_content_descriptions::Model>,
) {
    let (hash_map, descs, prof_descs) = model.into_iter().fold(
        (HashMap::new(), vec![], vec![]),
        |(mut map, mut descs, mut prof_descs), w| {
            let (ent, opt) = parsable_wrapper_from_model(profile_uuid, w.1);
            if let Some((desc, prof_desc)) = opt {
                descs.push(desc);
                prof_descs.push(prof_desc);
            }
            map.insert(w.0, ent);
            (map, descs, prof_descs)
        },
    );

    (
        serde_json::ser::to_string(&hash_map).unwrap(),
        descs,
        prof_descs,
    )
}

fn parsable_wrapper_from_model(
    profile_uuid: ProfileUuid,
    model: ModelParsableWrapper,
) -> (
    ParsableWrapper,
    Option<(
        entities::content_description::Model,
        entities::profile_content_descriptions::Model,
    )>,
) {
    let container = match &model {
        ModelParsableWrapper::Description(columns::other::Description(
            desc,
        ))
        | ModelParsableWrapper::Special(columns::other::Special(_, desc)) => {
            Some(description_from_model(profile_uuid, desc.clone()))
        }
        _ => None,
    };
    (ParsableWrapper::from(model), container)
}

fn description_from_model(
    profile_uuid: ProfileUuid,
    ModelContentDescription {
        uuid,
        description,
        datetime_created,
    }: ModelContentDescription,
) -> (
    entities::content_description::Model,
    entities::profile_content_descriptions::Model,
) {
    (
        entities::content_description::Model {
            uuid,
            description,
            datetime_created: datetime_to_str(datetime_created),
        },
        entities::profile_content_descriptions::Model {
            content_uuid: uuid,
            profile_uuid,
        },
    )
}
