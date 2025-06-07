use hermes::container::manual;
use itertools::Itertools;
use sea_orm::{EntityTrait, QueryTrait};

use crate::{
    db::{
        datetime_to_str,
        entities::{self, prelude::DataGroups},
        VecIntoActiveModel,
    },
    model::group::ModelGroup,
};

pub trait GroupsQuery {
    fn insert_many_query(
        groups: Vec<ModelGroup>,
    ) -> impl QueryTrait + Send + 'static {
        let models = groups
            .into_iter()
            .map(group_from_model)
            .collect_vec()
            .into_active_model_vec();
        DataGroups::insert_many(models)
    }
}

impl GroupsQuery for manual::Container<ModelGroup> {}

pub(super) fn group_from_model(
    ModelGroup {
        uuid,
        datetime_created,
    }: ModelGroup,
) -> entities::data_groups::Model {
    entities::data_groups::Model {
        uuid,
        datetime_created: datetime_to_str(datetime_created),
    }
}
