use std::sync::Arc;

use sqlx::{
    query_builder::Separated, sqlite::SqliteQueryResult, Encode, Execute, Pool, QueryBuilder,
    Sqlite, Type,
};

const MAX_PER_INSERT: usize = 10_000;
const MAX_PER_INSERT_F32: f32 = MAX_PER_INSERT as f32;

pub async fn insert_values<'args, Value>(
    pool: Arc<Pool<Sqlite>>,
    insert_query: &str,
    mut values: Vec<Value>,
    mut push_fn: impl FnMut(Separated<'_, 'args, Sqlite, &'static str>, Value),
) -> Result<SqliteQueryResult, ()> {
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
    println!("{}", query.sql());

    query.execute(&*pool).await.or(Err(()))
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
