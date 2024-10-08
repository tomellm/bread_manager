use std::sync::Arc;

use data_communicator::buffered::{change::ChangeResult, query::{self, QueryResponse}, storage::{self, Storage}, GetKey};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use crate::model::linker::Link;

use super::{utils, IntoChangeResult, MapToQueryError};

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct DbLink {
    pub uuid: Uuid,
    pub negative: Uuid,
    pub positive: Uuid,
}

impl GetKey<Uuid> for Link {
    fn key(&self) -> &Uuid {
        &self.uuid
    }
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

pub struct DbLinks {
    pool: Arc<Pool<Sqlite>>,
}

impl DbLinks {
    pub fn pool(&self) -> Arc<Pool<Sqlite>> {
        self.pool.clone()
    }
}

pub struct DbLinkInitArgs {
    pub pool: Arc<Pool<Sqlite>>,
    pub drop: bool,
}

impl From<(&Arc<Pool<Sqlite>>, bool)> for DbLinkInitArgs {
    fn from(value: (&Arc<Pool<Sqlite>>, bool)) -> Self {
        Self {
            pool: value.0.clone(),
            drop: value.1,
        }
    }
}

impl Storage<Uuid, Link> for DbLinks {
    type InitArgs = DbLinkInitArgs;
    fn init(
        DbLinkInitArgs { pool, drop }: Self::InitArgs,
    ) -> impl storage::InitFuture<Self> {
        async move {
            if drop {
                let _ = sqlx::query!("drop table if exists links;")
                    .execute(&*pool)
                    .await
                    .unwrap();
            }

            sqlx::query!(
                r#"
                create table if not exists links (
                    uuid blob primary key not null,
                    negative blob not null,
                    positive blob not null
                );
                "#
            )
            .execute(&*pool)
            .await
            .unwrap();

            Self { pool }
        }
    }
    fn insert(&mut self, value: &Link) -> impl storage::Future<ChangeResult> {
        let pool = self.pool();
        let link = DbLink::from(value);
        async move {
            sqlx::query!(
                r#"insert into links(
                    uuid, negative, positive
                ) values(?, ?, ?)"#,
                link.uuid,
                link.negative,
                link.positive
            )
            .execute(&*pool)
            .await
            .into_change_result()
        }
    }
    fn insert_many(&mut self, values: &[Link]) -> impl storage::Future<ChangeResult> {
        let pool = self.pool();
        let links = values.iter().map(DbLink::from).collect::<Vec<_>>();
        async move {
            utils::insert_values(
                pool,
                "insert into links(
                    uuid, negative, positive
                )",
                links,
                |mut builder, value| {
                    builder
                        .push_bind(value.uuid)
                        .push_bind(value.negative)
                        .push_bind(value.positive);
                },
            )
            .await
            .into_change_result()
        }
    }
    fn update(&mut self, value: &Link) -> impl storage::Future<ChangeResult> {
        let pool = self.pool();
        let link = DbLink::from(value);
        async move {
            sqlx::query!(
                r#"insert into links(
                    uuid, negative, positive
                ) values(?, ?, ?)"#,
                link.uuid,
                link.negative,
                link.positive
            )
            .execute(&*pool)
            .await
            .into_change_result()
        }
    }
    fn update_many(&mut self, values: &[Link]) -> impl storage::Future<ChangeResult> {
        let pool = self.pool();
        let links = values.iter().map(DbLink::from).collect::<Vec<_>>();
        async move {
            utils::transactional_execute_queries(
                pool,
                r#"
                update
                    links
                set
                    negative = ?,
                    positive = ?
                where
                    uuid = ?
                "#,
                links,
                |builder, value| {
                    builder
                        .bind(value.negative)
                        .bind(value.positive)
                        .bind(value.uuid)
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
            sqlx::query!("delete from links where uuid = ?", key)
                .execute(&*pool)
                .await
                .into_change_result()
        }
    }
    fn delete_many(&mut self, keys: &[Uuid]) -> impl storage::Future<ChangeResult> {
        let pool = self.pool();
        let keys = keys.to_vec();
        async move {
            utils::add_in_items(
                "delete from links where uuid in (",
                keys.iter(),
                ")",
            )
            .build()
            .execute(&*pool)
            .await
            .into_change_result()
        }
    }

    fn get_all(&mut self) -> impl storage::Future<QueryResponse<Uuid, Link>> {
        let pool = self.pool();
        async move {
            sqlx::query_as!(
                DbLink,
                r#"
                select
                    uuid as "uuid: uuid::Uuid",
                    negative as "negative: uuid::Uuid",
                    positive as "positive: uuid::Uuid"
                from
                    links
                "#
            )
            .fetch_all(&*pool)
            .await
            .map(|vals| {
                vals.into_iter()
                    .map(Into::<Link>::into)
                    .collect::<Vec<_>>()
            })
            .map_query_error()
            .into()
        }
    }
    fn get_by_id(&mut self, key: Uuid) -> impl storage::Future<QueryResponse<Uuid, Link>> {
        let pool = self.pool();
        async move {
            sqlx::query_as!(
                DbLink,
                r#"
                select
                    uuid as "uuid: uuid::Uuid",
                    negative as "negative: uuid::Uuid",
                    positive as "positive: uuid::Uuid"
                from
                    links
                where
                    uuid = ?
                "#,
                key
            )
            .fetch_one(&*pool)
            .await
            .map(Into::<Link>::into)
            .map_query_error()
            .into()
        }
    }
    fn get_by_ids(
        &mut self,
        keys: Vec<Uuid>,
    ) -> impl storage::Future<QueryResponse<Uuid, Link>> {
        let pool = self.pool();
        async move {
            utils::add_in_items(
                r#"
                select
                    uuid as "uuid: uuid::Uuid",
                    negative as "negative: uuid::Uuid",
                    positive as "positive: uuid::Uuid"
                from
                    links
                where
                    uuid in (
                "#,
                keys.iter(),
                ")",
            )
            .build_query_as::<DbLink>()
            .fetch_all(&*pool)
            .await
            .map(|vals| {
                vals.into_iter()
                    .map(Into::<Link>::into)
                    .collect::<Vec<_>>()
            })
            .map_query_error()
            .into()
        }
    }
    fn get_by_predicate(
        &mut self,
        predicate: query::Predicate<Link>,
    ) -> impl storage::Future<QueryResponse<Uuid, Link>> {
        let pool = self.pool();
        async move {
            sqlx::query_as!(
                DbLink,
                r#"
                select
                    uuid as "uuid: uuid::Uuid",
                    negative as "negative: uuid::Uuid",
                    positive as "positive: uuid::Uuid"
                from
                    links
                "#
            )
            .fetch_all(&*pool)
            .await
            .map(|vals| {
                vals.into_iter()
                    .filter_map(|val| {
                        let possible_link = val.into();
                        if predicate(&possible_link) {
                            Some(possible_link)
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
