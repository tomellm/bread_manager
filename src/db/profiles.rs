use std::sync::Arc;

use diesel::prelude::*;

use crate::{model::profiles::Profile, schema};

use super::Uuid;

pub static PROFILES_FROM_DB_FN: Arc<dyn Fn(DbProfile) -> Profile + Sync + Send + 'static> =
    Arc::new(|val: DbProfile| val.into_profile());

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = schema::profiles)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DbProfile {
    uuid: Uuid,
    name: String,
    origin_name: String,
    data: Vec<u8>,
}

impl DbProfile {
    pub fn from_profile(profile: &Profile) -> Self {
        let (uuid, name, origin_name, data) = profile.to_db();
        Self {
            uuid,
            name,
            origin_name,
            data,
        }
    }

    pub fn into_profile(self) -> Profile {
        Profile::from_db(self.uuid, self.name, self.origin_name, &self.data)
    }
}
