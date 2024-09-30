use std::sync::Arc;

use super::{utils, IntoChangeResult, MapToQueryError};
use data_communicator::buffered::{
    change::ChangeResult,
    query::{self, QueryResponse},
    storage::{self, Storage},
    GetKey,
};
use sqlx::{types::Uuid, Pool, Sqlite};

use crate::model::profiles::Profile;

#[derive(sqlx::FromRow)]
struct DbProfile {
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

impl GetKey<Uuid> for Profile {
    fn key(&self) -> &Uuid {
        &self.uuid
    }
}

pub struct DbProfiles {
    pool: Arc<Pool<Sqlite>>,
}

impl DbProfiles {
    pub fn pool(&self) -> Arc<Pool<Sqlite>> {
        self.pool.clone()
    }
}

pub struct DbProfilesInitArgs {
    pub pool: Arc<Pool<Sqlite>>,
    pub drop: bool,
}

impl From<(&Arc<Pool<Sqlite>>, bool)> for DbProfilesInitArgs {
    fn from(value: (&Arc<Pool<Sqlite>>, bool)) -> Self {
        Self { pool: value.0.clone(), drop: value.1 }
    }
}

impl Storage<Uuid, Profile> for DbProfiles {
    type InitArgs = DbProfilesInitArgs;
    fn init(DbProfilesInitArgs { pool, drop }: Self::InitArgs) -> impl storage::InitFuture<Self> {
        async move {
            if drop {
                let _ = sqlx::query!("drop table if exists profiles;")
                    .execute(&*pool)
                    .await
                    .unwrap();
            }

            let _ = sqlx::query!(
                r#"
                create table if not exists profiles (
                    uuid blob primary key not null,
                    name text not null,
                    origin_name text not null,
                    data blob not null
                );
                "#
            )
            .execute(&*pool)
            .await
            .unwrap();

            Self { pool }
        }
    }
    fn update(&mut self, value: &Profile) -> impl storage::Future<ChangeResult> {
        let pool = self.pool();
        let profile = DbProfile::from_profile(value);
        async move {
            sqlx::query!(
                "insert into profiles(uuid, name, origin_name, data) values(?, ?, ?, ?)",
                profile.uuid,
                profile.name,
                profile.origin_name,
                profile.data
            )
            .execute(&*pool)
            .await
            .into_change_result()
        }
    }
    fn update_many(&mut self, values: &[Profile]) -> impl storage::Future<ChangeResult> {
        let pool = self.pool();
        let profiles = values
            .iter()
            .map(DbProfile::from_profile)
            .collect::<Vec<_>>();
        async move {
            utils::insert_values(
                pool,
                "insert into profiles(uuid, name, data)",
                profiles,
                |mut builder, value| {
                    builder
                        .push_bind(value.uuid)
                        .push_bind(value.name)
                        .push_bind(value.data);
                },
            )
            .await
            .into_change_result()
        }
    }
    fn delete(&mut self, key: &Uuid) -> impl storage::Future<ChangeResult> {
        let pool = self.pool();
        let key = *key;
        async move {
            sqlx::query!("delete from profiles where uuid = ?", key)
                .execute(&*pool)
                .await
                .into_change_result()
        }
    }
    fn delete_many(&mut self, keys: &[Uuid]) -> impl storage::Future<ChangeResult> {
        let pool = self.pool();
        let keys = keys.to_vec();
        async move {
            utils::add_in_items("delete from profiles where uuid in (", keys.iter(), ")")
                .build()
                .execute(&*pool)
                .await
                .into_change_result()
        }
    }
    fn get_all(&mut self) -> impl storage::Future<QueryResponse<Uuid, Profile>> {
        let pool = self.pool();
        async move {
            sqlx::query_as!(
                DbProfile,
                r#"select uuid as "uuid: uuid::Uuid", name, origin_name, data from profiles"#
            )
            .fetch_all(&*pool)
            .await
            .map(|vals| {
                vals.into_iter()
                    .map(DbProfile::into_profile)
                    .collect::<Vec<_>>()
            })
            .map_query_error()
            .into()
        }
    }
    fn get_by_id(&mut self, key: Uuid) -> impl storage::Future<QueryResponse<Uuid, Profile>> {
        let pool = self.pool();
        async move {
            sqlx::query_as!(
                DbProfile,
                r#"
                select
                    uuid as "uuid: uuid::Uuid", name, origin_name, data
                from
                    profiles
                where
                    uuid = ?
                "#,
                key
            )
            .fetch_one(&*pool)
            .await
            .map(DbProfile::into_profile)
            .map_query_error()
            .into()
        }
    }
    fn get_by_ids(
        &mut self,
        keys: Vec<Uuid>,
    ) -> impl storage::Future<QueryResponse<Uuid, Profile>> {
        let pool = self.pool();
        async move {
            utils::add_in_items(
                r#"
                select
                    uuid as "uuid: uuid::Uuid", name, origin_name, data
                from
                    profiles
                where
                    uuid in (
                "#,
                keys.iter(),
                ")",
            )
            .build_query_as::<DbProfile>()
            .fetch_all(&*pool)
            .await
            .map(|vals| {
                vals.into_iter()
                    .map(DbProfile::into_profile)
                    .collect::<Vec<_>>()
            })
            .map_query_error()
            .into()
        }
    }
    fn get_by_predicate(
        &mut self,
        predicate: query::Predicate<Profile>,
    ) -> impl storage::Future<QueryResponse<Uuid, Profile>> {
        let pool = self.pool();
        async move {
            sqlx::query_as!(
                DbProfile,
                r#"
                select
                    uuid as "uuid: uuid::Uuid", name, origin_name, data
                from
                    profiles
                "#
            )
            .fetch_all(&*pool)
            .await
            .map(|vals| {
                vals.into_iter()
                    .filter_map(|val| {
                        let profile = val.into_profile();
                        if predicate(&profile) {
                            Some(profile)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .map_query_error()
            .into()
        }
    }
}
