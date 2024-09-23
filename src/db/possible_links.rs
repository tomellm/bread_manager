use std::sync::Arc;

use futures::future::BoxFuture;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use crate::{
    model::linker::PossibleLink,
    utils::{
        changer::{ActionType, Response},
        communicator::{GetKey, Storage},
    },
};

use super::{error_to_response, utils};

pub struct DbPossibleLinks {
    pub(super) pool: Arc<Pool<Sqlite>>,
}

#[derive(Clone, Debug)]
struct DbPossibleLink {
    uuid: Uuid,
    negative: Uuid,
    positive: Uuid,
    probability: f64,
}

impl GetKey<Uuid> for PossibleLink {
    fn get_key(&self) -> Uuid {
        self.uuid
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

impl Storage<Uuid, PossibleLink> for DbPossibleLinks {
    fn get_all(&mut self) -> BoxFuture<'static, Response<Uuid, PossibleLink>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let links = sqlx::query_as!(
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
            .unwrap()
            .into_iter()
            .map(Into::<PossibleLink>::into)
            .collect();
            let action = ActionType::GetAll(links);
            Response::ok(&action)
        })
    }
    fn set(&mut self, value: PossibleLink) -> BoxFuture<'static, Response<Uuid, PossibleLink>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let link = DbPossibleLink::from(&value);
            let query_result = sqlx::query!(
                r#"insert into possible_links(
                    uuid, negative, positive, probability
                ) values(?, ?, ?, ?)"#,
                link.uuid,
                link.negative,
                link.positive,
                link.probability
            )
            .execute(&*pool)
            .await;
            error_to_response(query_result, &ActionType::Set(value))
        })
    }
    fn set_many(
        &mut self,
        values: Vec<PossibleLink>,
    ) -> BoxFuture<'static, Response<Uuid, PossibleLink>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let links = values.iter().map(DbPossibleLink::from).collect::<Vec<_>>();

            let query_result = utils::insert_values(
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
            .await;

            Response::from_result(query_result, &ActionType::SetMany(values))
        })
    }
    fn update(&mut self, value: PossibleLink) -> BoxFuture<'static, Response<Uuid, PossibleLink>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let link = DbPossibleLink::from(&value);
            let query_result = sqlx::query!(
                r#"insert into possible_links(
                    uuid, negative, positive, probability
                ) values(?, ?, ?, ?)"#,
                link.uuid,
                link.negative,
                link.positive,
                link.probability,
            )
            .execute(&*pool)
            .await;
            error_to_response(query_result, &ActionType::Update(value))
        })
    }
    fn update_many(
        &mut self,
        values: Vec<PossibleLink>,
    ) -> BoxFuture<'static, Response<Uuid, PossibleLink>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let links = values.iter().map(DbPossibleLink::from).collect::<Vec<_>>();

            let query_result = utils::insert_values(
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
            .await;

            Response::from_result(query_result, &ActionType::SetMany(values))
        })
    }
    fn delete(&mut self, key: Uuid) -> BoxFuture<'static, Response<Uuid, PossibleLink>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let query_result = sqlx::query!("delete from possible_links where uuid = ?", key)
                .execute(&*pool)
                .await;
            error_to_response(query_result, &ActionType::Delete(key))
        })
    }
    fn delete_many(&mut self, keys: Vec<Uuid>) -> BoxFuture<'static, Response<Uuid, PossibleLink>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let query_result = utils::add_in_items(
                "delete from possible_links where uuid in (",
                keys.iter(),
                ")",
            )
            .build()
            .execute(&*pool)
            .await;

            Response::from_result(query_result, &ActionType::DeleteMany(keys))
        })
    }
    fn setup(&mut self, drop: bool) -> BoxFuture<'static, Result<Vec<PossibleLink>, ()>> {
        //
        let pool = self.pool.clone();
        Box::pin(async move {
            if drop {
                let _ = sqlx::query!("drop table if exists possible_links;")
                    .execute(&*pool)
                    .await
                    .map_err(|_| ())?;
            }

            let _ = sqlx::query!(
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
            .map_err(|_| ())?;

            Ok(sqlx::query_as!(
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
            .unwrap()
            .into_iter()
            .map(DbPossibleLink::into)
            .collect())
        })
    }
}
