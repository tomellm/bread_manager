use std::sync::Arc;

use data_communicator::buffered::{
    change::ChangeResult,
    query::{self, QueryResponse},
    storage::{self, Storage},
    GetKey,
};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use crate::model::linker::PossibleLink;

use super::{utils, IntoChangeResult, MapToQueryError};

#[derive(Clone, Debug, sqlx::FromRow)]
struct DbPossibleLink {
    uuid: Uuid,
    negative: Uuid,
    positive: Uuid,
    probability: f64,
}

impl GetKey<Uuid> for PossibleLink {
    fn key(&self) -> &Uuid {
        &self.uuid
    }
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

pub struct DbPossibleLinks {
    pool: Arc<Pool<Sqlite>>,
}

impl DbPossibleLinks {
    pub fn pool(&self) -> Arc<Pool<Sqlite>> {
        self.pool.clone()
    }
}

pub struct DbPossibleLinkInitArgs {
    pub pool: Arc<Pool<Sqlite>>,
    pub drop: bool,
}

impl From<(&Arc<Pool<Sqlite>>, bool)> for DbPossibleLinkInitArgs {
    fn from(value: (&Arc<Pool<Sqlite>>, bool)) -> Self {
        Self {
            pool: value.0.clone(),
            drop: value.1,
        }
    }
}

impl Storage<Uuid, PossibleLink> for DbPossibleLinks {
    type InitArgs = DbPossibleLinkInitArgs;
    fn init(
        DbPossibleLinkInitArgs { pool, drop }: Self::InitArgs,
    ) -> impl storage::InitFuture<Self> {
        async move {
            if drop {
                let _ = sqlx::query!("drop table if exists possible_links;")
                    .execute(&*pool)
                    .await
                    .unwrap();
            }

            sqlx::query!(
                r#"
                create table if not exists possible_links (
                    uuid blob primary key not null,
                    negative blob not null,
                    positive blob not null,
                    probability real not null
                );
                "#
            )
            .execute(&*pool)
            .await
            .unwrap();

            Self { pool }
        }
    }
    fn insert(&mut self, value: &PossibleLink) -> impl storage::Future<ChangeResult> {
        let pool = self.pool();
        let link = DbPossibleLink::from(value);
        async move {
            sqlx::query!(
                r#"insert into possible_links(
                    uuid, negative, positive, probability
                ) values(?, ?, ?, ?)"#,
                link.uuid,
                link.negative,
                link.positive,
                link.probability
            )
            .execute(&*pool)
            .await
            .into_change_result()
        }
    }
    fn insert_many(&mut self, values: &[PossibleLink]) -> impl storage::Future<ChangeResult> {
        let pool = self.pool();
        let links = values.iter().map(DbPossibleLink::from).collect::<Vec<_>>();
        async move {
            utils::insert_values(
                pool,
                "insert into possible_links(
                    uuid, negative, positive, probability
                )",
                links,
                |mut builder, value| {
                    builder
                        .push_bind(value.uuid)
                        .push_bind(value.negative)
                        .push_bind(value.positive)
                        .push_bind(value.probability);
                },
            )
            .await
            .into_change_result()
        }
    }
    fn update(&mut self, value: &PossibleLink) -> impl storage::Future<ChangeResult> {
        let pool = self.pool();
        let link = DbPossibleLink::from(value);
        async move {
            sqlx::query!(
                r#"insert into possible_links(
                    uuid, negative, positive, probability
                ) values(?, ?, ?, ?)"#,
                link.uuid,
                link.negative,
                link.positive,
                link.probability
            )
            .execute(&*pool)
            .await
            .into_change_result()
        }
    }
    fn update_many(&mut self, values: &[PossibleLink]) -> impl storage::Future<ChangeResult> {
        let pool = self.pool();
        let links = values.iter().map(DbPossibleLink::from).collect::<Vec<_>>();
        async move {
            utils::transactional_execute_queries(
                pool,
                r#"
                update
                    possible_links
                set
                    negative = ?,
                    positive = ?,
                    probability = ?
                where
                    uuid = ?
                "#,
                links,
                |builder, value| {
                    builder
                        .bind(value.negative)
                        .bind(value.positive)
                        .bind(value.probability)
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
            sqlx::query!("delete from possible_links where uuid = ?", key)
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
                "delete from possible_links where uuid in (",
                keys.iter(),
                ")",
            )
            .build()
            .execute(&*pool)
            .await
            .into_change_result()
        }
    }

    fn get_all(&mut self) -> impl storage::Future<QueryResponse<Uuid, PossibleLink>> {
        let pool = self.pool();
        async move {
            sqlx::query_as!(
                DbPossibleLink,
                r#"
                select
                    uuid as "uuid: uuid::Uuid",
                    negative as "negative: uuid::Uuid",
                    positive as "positive: uuid::Uuid",
                    probability
                from
                    possible_links
                "#
            )
            .fetch_all(&*pool)
            .await
            .map(|vals| {
                vals.into_iter()
                    .map(Into::<PossibleLink>::into)
                    .collect::<Vec<_>>()
            })
            .map_query_error()
            .into()
        }
    }
    fn get_by_id(&mut self, key: Uuid) -> impl storage::Future<QueryResponse<Uuid, PossibleLink>> {
        let pool = self.pool();
        async move {
            sqlx::query_as!(
                DbPossibleLink,
                r#"
                select
                    uuid as "uuid: uuid::Uuid",
                    negative as "negative: uuid::Uuid",
                    positive as "positive: uuid::Uuid",
                    probability
                from
                    possible_links
                where
                    uuid = ?
                "#,
                key
            )
            .fetch_one(&*pool)
            .await
            .map(Into::<PossibleLink>::into)
            .map_query_error()
            .into()
        }
    }
    fn get_by_ids(
        &mut self,
        keys: Vec<Uuid>,
    ) -> impl storage::Future<QueryResponse<Uuid, PossibleLink>> {
        let pool = self.pool();
        async move {
            utils::add_in_items(
                r#"
                select
                    uuid as "uuid: uuid::Uuid",
                    negative as "negative: uuid::Uuid",
                    positive as "positive: uuid::Uuid",
                    probability
                from
                    possible_links
                where
                    uuid in (
                "#,
                keys.iter(),
                ")",
            )
            .build_query_as::<DbPossibleLink>()
            .fetch_all(&*pool)
            .await
            .map(|vals| {
                vals.into_iter()
                    .map(Into::<PossibleLink>::into)
                    .collect::<Vec<_>>()
            })
            .map_query_error()
            .into()
        }
    }
    fn get_by_predicate(
        &mut self,
        predicate: query::Predicate<PossibleLink>,
    ) -> impl storage::Future<QueryResponse<Uuid, PossibleLink>> {
        let pool = self.pool();
        async move {
            sqlx::query_as!(
                DbPossibleLink,
                r#"
                select
                    uuid as "uuid: uuid::Uuid",
                    negative as "negative: uuid::Uuid",
                    positive as "positive: uuid::Uuid",
                    probability
                from
                    possible_links
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
