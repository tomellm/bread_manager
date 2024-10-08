use std::sync::Arc;

use sqlx::{
    query::Query, query_builder::Separated, sqlite::SqliteQueryResult, Database, Encode, Execute,
    Pool, QueryBuilder, Sqlite, Type,
};
use tracing::trace;

const MAX_PER_INSERT: usize = 10_000;
const MAX_PER_INSERT_F32: f32 = MAX_PER_INSERT as f32;

pub async fn insert_values<'args, Value>(
    pool: Arc<Pool<Sqlite>>,
    insert_query: &str,
    mut values: Vec<Value>,
    mut push_fn: impl FnMut(Separated<'_, 'args, Sqlite, &'static str>, Value),
) -> Result<SqliteQueryResult, sqlx::Error> {
    let mut builder = QueryBuilder::<Sqlite>::new(insert_query);

    loop {
        let mut break_out = false;
        let to_drain = if values.len() >= MAX_PER_INSERT {
            MAX_PER_INSERT
        } else {
            break_out = true;
            values.len()
        };
        let chunk = values.drain(..to_drain).collect::<Vec<_>>();
        builder.push_values(chunk, |builder, value| {
            push_fn(builder, value);
        });

        if break_out {
            break;
        }
    }

    let query = builder.build();
    trace!(msg = format!("{}", query.sql()));

    query.execute(&*pool).await
}

pub async fn transactional_execute_queries<'args, Value>(
    pool: Arc<Pool<Sqlite>>,
    query: &'args str,
    values: Vec<Value>,
    mut push_fn: impl FnMut(
        Query<'args, Sqlite, <Sqlite as Database>::Arguments<'args>>,
        Value,
    ) -> Query<'args, Sqlite, <Sqlite as Database>::Arguments<'args>>,
) -> Result<(), sqlx::Error> {
    let mut connection = pool.acquire().await?;
    sqlx::query!("begin transaction;").execute(&mut *connection).await?;

    for value in values {
        let builder = sqlx::query(query);
        let query_result = push_fn(builder, value).execute(&mut *connection).await;
        if query_result.is_err() {
            sqlx::query!("rollback").execute(&mut *connection).await?;
            return query_result.map(|_| ());
        }
    }

    sqlx::query!("commit;").execute(&mut *connection).await?;

    Ok(())
}

pub fn add_in_items<'args, I, T>(
    query_front: &str,
    items: I,
    query_back: &str,
) -> QueryBuilder<'args, Sqlite>
where
    I: Iterator<Item = T>,
    T: 'args + Encode<'args, Sqlite> + Send + Type<Sqlite>,
{
    let mut query_builder: QueryBuilder<'args, Sqlite> = QueryBuilder::new(query_front);

    items.enumerate().for_each(|(index, id)| {
        if index != 0 {
            query_builder.push(",");
        };
        query_builder.push_bind(id);
    });

    query_builder.push(query_back);

    query_builder
}
