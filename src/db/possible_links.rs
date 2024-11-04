use std::sync::Arc;

use diesel::{
    prelude::{Insertable, Queryable},
    Selectable,
};

use crate::{model::linker::PossibleLink, schema};

use super::Uuid;

pub static POSSIBLE_LINK_FROM_DB_FN: Arc<
    dyn Fn(DbPossibleLink) -> PossibleLink + Sync + Send + 'static,
> = Arc::new(|val: DbPossibleLink| PossibleLink::from(val));

#[derive(Clone, Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = schema::possible_links)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(treat_none_as_default_value = false)]
pub(crate) struct DbPossibleLink {
    uuid: Uuid,
    negative: Uuid,
    positive: Uuid,
    probability: f64,
}

impl From<&PossibleLink> for DbPossibleLink {
    fn from(value: &PossibleLink) -> Self {
        Self {
            uuid: value.uuid,
            negative: *value.negative,
            positive: *value.positive,
            probability: value.probability,
        }
    }
}

impl From<DbPossibleLink> for PossibleLink {
    fn from(value: DbPossibleLink) -> Self {
        PossibleLink {
            uuid: value.uuid,
            negative: value.negative.into(),
            positive: value.positive.into(),
            probability: value.probability,
        }
    }
}
