use std::sync::Arc;

use super::utils;
use futures::future::BoxFuture;
use sqlx::{types::Uuid, Pool, Sqlite};

use crate::{
    model::profiles::Profile,
    utils::{
        changer::{ActionType, Response},
        communicator::{GetKey, Storage},
    },
};

use super::error_to_response;

pub struct ProfilesDB {
    pub(super) pool: Arc<Pool<Sqlite>>,
}

struct DbProfile {
    uuid: Uuid,
    name: String,
    data: Vec<u8>,
}

impl DbProfile {
    pub fn from_profile(profile: &Profile) -> Self {
        let (uuid, name, data) = profile.to_db();
        Self {
            uuid,
            name,
            data,
        }
    }

    pub fn to_profile(self) -> Profile {
        Profile::from_db(self.uuid, self.name, self.data)
    }
}

impl GetKey<Uuid> for Profile {
    fn get_key(&self) -> Uuid {
        self.uuid
    }
}

impl Storage<Uuid, Profile> for ProfilesDB {
    fn get_all(&self) -> BoxFuture<'static, Response<Uuid, Profile>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let profiles = sqlx::query_as!(
                DbProfile,
                r#"select uuid as "uuid: uuid::Uuid", name, data from profiles"#
            )
            .fetch_all(&*pool)
            .await
            .unwrap()
            .into_iter()
            .map(DbProfile::to_profile)
            .collect();

            let action = ActionType::GetAll(profiles);
            Response::ok(&action)
        })
    }
    fn set(&self, value: Profile) -> BoxFuture<'static, Response<Uuid, Profile>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let profile = DbProfile::from_profile(&value);
            let query_result = sqlx::query!(
                "insert into profiles values(?, ?, ?)",
                profile.uuid,
                profile.name,
                profile.data
            )
            .execute(&*pool)
            .await;
            error_to_response(query_result, ActionType::Set(value))
        })
    }
    fn set_many(&self, values: Vec<Profile>) -> BoxFuture<'static, Response<Uuid, Profile>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let profiles = values
                .iter()
                .map(DbProfile::from_profile)
                .collect::<Vec<_>>();

            let query_result = utils::insert_values(
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
            .await;

            Response::from_result(query_result, ActionType::SetMany(values))
        })
    }
    fn update(&self, value: Profile) -> BoxFuture<'static, Response<Uuid, Profile>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let profile = DbProfile::from_profile(&value);
            let query_result = sqlx::query!(
                "insert into profiles values(?, ?, ?)",
                profile.uuid,
                profile.name,
                profile.data
            )
            .execute(&*pool)
            .await;
            Response::from_result(query_result, ActionType::Update(value))
        })
    }
    fn update_many(&self, values: Vec<Profile>) -> BoxFuture<'static, Response<Uuid, Profile>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let profiles = values
                .iter()
                .map(DbProfile::from_profile)
                .collect::<Vec<_>>();

            let query_result = utils::insert_values(
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
            .await;

            Response::from_result(query_result, ActionType::SetMany(values))
        })
    }
    fn delete(&self, key: Uuid) -> BoxFuture<'static, Response<Uuid, Profile>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let query_result = sqlx::query!("delete from profiles where uuid = ?", key)
                .execute(&*pool)
                .await;
            Response::from_result(query_result, ActionType::Delete(key))
        })
    }
    fn delete_many(&self, keys: Vec<Uuid>) -> BoxFuture<'static, Response<Uuid, Profile>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let query_result =
                utils::add_in_items("delete from profiles where uuid in (", keys.iter(), ")")
                    .build()
                    .execute(&*pool)
                    .await;
            Response::from_result(query_result, ActionType::DeleteMany(keys))
        })
    }
    fn setup(&self, drop: bool) -> BoxFuture<'static, Result<Vec<Profile>, ()>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            if drop {
                let _ = sqlx::query!("drop table if exists profiles;")
                    .execute(&*pool)
                    .await
                    .map_err(|_| ())?;
            }

            let _ = sqlx::query!(
                r#"
                create table if not exists profiles (
                    uuid blob primary key not null,
                    name text not null,
                    data blob not null
                );
                "#
            )
            .execute(&*pool)
            .await
            .map_err(|_| ())?;

            Ok(sqlx::query_as!(
                DbProfile,
                r#"select uuid as "uuid: uuid::Uuid", name, data from profiles"#
            )
            .fetch_all(&*pool)
            .await
            .unwrap()
            .into_iter()
            .map(DbProfile::to_profile)
            .collect())
        })
    }
}
