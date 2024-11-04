use std::sync::Arc;

use diesel::{prelude::{Insertable, Queryable}, Selectable};

use crate::{model::linker::Link, schema};

use super::Uuid;

pub static LINK_FROM_DB_FN: Arc<
    dyn Fn(DbLink) -> Link + Sync + Send + 'static,
> = Arc::new(|val: DbLink| Link::from(val));

#[derive(Clone, Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = schema::links)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DbLink {
    pub uuid: Uuid,
    pub negative: Uuid,
    pub positive: Uuid,
}

impl From<&Link> for DbLink {
    fn from(value: &Link) -> Self {
        Self {
            uuid: value.uuid,
            negative: *value.negative,
            positive: *value.positive,
        }
    }
}

impl From<DbLink> for Link {
    fn from(value: DbLink) -> Self {
        Link {
            uuid: value.uuid,
            negative: value.negative.into(),
            positive: value.positive.into(),
        }
    }
}
