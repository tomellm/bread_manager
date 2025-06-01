use std::{collections::HashMap, hash::Hash};

use chrono::{DateTime, Local, NaiveDate, NaiveTime};
use itertools::Itertools;
use sea_orm::{ActiveModelTrait, IntoActiveModel};
use uuid::Uuid;

pub mod builders;
mod entities;
pub mod query;

pub const DATE_FORMAT: &str = "%Y-%m-%d";
pub const TIME_FORMAT: &str = "%H:%M:%S";
pub const TZ_FORMAT: &str = "%z";

fn datetime_format() -> String {
    format!("{DATE_FORMAT} {TIME_FORMAT} {TZ_FORMAT}")
}

fn parse_datetime_str(to_parse: &str) -> DateTime<Local> {
    DateTime::parse_from_str(to_parse, &datetime_format())
        .unwrap()
        .with_timezone(&Local)
}

fn datetime_to_str(datetime: DateTime<Local>) -> String {
    datetime.format(&datetime_format()).to_string()
}

fn parse_naive_date_str(to_parse: &str) -> NaiveDate {
    NaiveDate::parse_from_str(to_parse, DATE_FORMAT).unwrap()
}

fn naive_date_to_str(date: NaiveDate) -> String {
    date.format(DATE_FORMAT).to_string()
}

fn parse_naive_time_str(to_parse: &str) -> NaiveTime {
    NaiveTime::parse_from_str(to_parse, TIME_FORMAT).unwrap()
}

fn naive_time_to_str(time: NaiveTime) -> String {
    time.format(TIME_FORMAT).to_string()
}

fn uuid_to_str(uuid: Uuid) -> String {
    uuid.hyphenated().to_string()
}

pub trait VecIntoActiveModel<T, A>
where
    T: IntoActiveModel<A>,
    A: ActiveModelTrait,
{
    fn into_active_model_vec(self) -> impl Iterator<Item = A>;
}

impl<T, A> VecIntoActiveModel<T, A> for Vec<T>
where
    T: IntoActiveModel<A>,
    A: ActiveModelTrait,
{
    fn into_active_model_vec(self) -> impl Iterator<Item = A> {
        self.into_iter().map(IntoActiveModel::into_active_model)
    }
}

pub trait InitUuid<T> {
    fn init() -> T;
}

impl<T> InitUuid<T> for T
where
    T: From<Uuid>,
{
    fn init() -> T {
        Uuid::new_v4().into()
    }
}

pub type DbUuid = sea_orm::entity::prelude::Uuid;

#[macro_export]
macro_rules! uuid_impls {
    ($type:ident) => {
        #[repr(transparent)]
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
            ::sea_orm::DeriveValueType,
        )]
        //#[sea_orm(value_type = "String")]
        pub struct $type(::uuid::Uuid);

        impl ::std::ops::Deref for $type {
            type Target = ::uuid::Uuid;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl From<::sea_orm::entity::prelude::Uuid> for $type {
            fn from(value: ::sea_orm::entity::prelude::Uuid) -> Self {
                Self(::uuid::Uuid::from_u128(value.as_u128()))
            }
        }

        //impl ::sea_query::Nullable for $type {
        //    fn null() -> ::sea_orm::Value {
        //        ::sea_orm::Value::String(None)
        //    }
        //}

        impl TryFrom<&str> for $type {
            type Error = ::sea_orm::DbErr;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                let uuid = ::uuid::Uuid::parse_str(value).map_err(|err| {
                    ::sea_orm::DbErr::Custom(format!(
                        "Uuid parsing Error: {err:?}"
                    ))
                })?;
                Ok(uuid.into())
            }
        }

        impl ::sea_orm::TryFromU64 for $type {
            fn try_from_u64(n: u64) -> Result<Self, sea_orm::DbErr> {
                Ok(::uuid::Uuid::from_u128(n as u128).into())
            }
        }
    };
}

pub fn combine_types<O, OId, I>(
    outer: Vec<O>,
    inner: Vec<I>,
    outer_id: impl Fn(&O) -> OId,
    outer_id_from_inner: impl Fn(&I) -> OId,
    feed_inner: impl Fn(&mut O, Vec<I>),
) -> Vec<O>
where
    OId: Eq + Hash,
{
    if inner.is_empty() {
        return outer;
    }

    let mut groups =
        inner
            .into_iter()
            .fold(HashMap::<OId, Vec<I>>::new(), |mut map, i| {
                map.entry(outer_id_from_inner(&i)).or_default().push(i);
                map
            });

    outer
        .into_iter()
        .map(|mut o| {
            let o_id = outer_id(&o);
            if let Some(i_v) = groups.remove(&o_id) {
                feed_inner(&mut o, i_v)
            }
            o
        })
        .collect_vec()
}
