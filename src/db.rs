pub mod link;
pub mod possible_links;
pub mod profiles;
pub mod records;
mod utils;

use std::sync::Arc;

use data_communicator::buffered::{
    change::{ChangeError, ChangeResult},
    communicator::Communicator,
    container::DataContainer,
    query::QueryError,
};
use link::DbLinks;
use possible_links::DbPossibleLinks;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteQueryResult},
    Error, SqlitePool,
};
use uuid::Uuid;

use crate::model::{linker::{Link, PossibleLink}, profiles::Profile, records::ExpenseRecord};

use self::{profiles::DbProfiles, records::DbRecords};

pub struct DB {
    profiles_container: DataContainer<Uuid, Profile, DbProfiles>,
    records_container: DataContainer<Uuid, ExpenseRecord, DbRecords>,
    links_container: DataContainer<Uuid, Link, DbLinks>,
    possible_links_container: DataContainer<Uuid, PossibleLink, DbPossibleLinks>,
}

impl DB {
    pub async fn get_db(drop: bool) -> Result<Self, ()> {
        let options = SqliteConnectOptions::new()
            .filename("db/bread_manager.sqlite3")
            .create_if_missing(true);

        let pool = Arc::new(SqlitePool::connect_with(options).await.map_err(|_| ())?);
        let profiles_container = DataContainer::new((&pool, drop).into()).await;
        let records_container = DataContainer::new((&pool, drop).into()).await;
        let links_container = DataContainer::new((&pool, drop).into()).await;
        let possible_links_container = DataContainer::new((&pool, drop).into()).await;
        Ok(Self {
            profiles_container,
            records_container,
            links_container,
            possible_links_container,
        })
    }

    pub fn profiles_signal(&mut self) -> Communicator<Uuid, Profile> {
        self.profiles_container.communicator()
    }

    pub fn records_signal(&mut self) -> Communicator<Uuid, ExpenseRecord> {
        self.records_container.communicator()
    }

    pub fn links_signal(&mut self) -> Communicator<Uuid, Link> {
        self.links_container.communicator()
    }

    pub fn possible_links_signal(&mut self) -> Communicator<Uuid, PossibleLink> {
        self.possible_links_container.communicator()
    }

    pub fn state_update(&mut self) {
        self.profiles_container.state_update();
        self.records_container.state_update();
        self.links_container.state_update();
        self.possible_links_container.state_update();
    }
}

pub trait IntoChangeResult {
    fn into_change_result(self) -> ChangeResult;
}

impl IntoChangeResult for Result<SqliteQueryResult, sqlx::Error> {
    fn into_change_result(self) -> ChangeResult {
        match self {
            Ok(_) => ChangeResult::Success,
            Err(err) => ChangeResult::Error(ChangeError::DatabaseError(format!("{err:?}"))),
        }
    }
}

impl IntoChangeResult for Result<(), sqlx::Error> {
    fn into_change_result(self) -> ChangeResult {
        match self {
            Ok(()) => ChangeResult::Success,
            Err(err) => ChangeResult::Error(ChangeError::DatabaseError(format!("{err:?}"))),
        }
    }
}

impl IntoChangeResult for Vec<ChangeResult> {
    fn into_change_result(self) -> ChangeResult {
        let (_, errors): (Vec<_>, Vec<_>) = self.into_iter().partition(|res| {
            match res {
                ChangeResult::Success => true,
                ChangeResult::Error(_) => false,
            }
        });

        if !errors.is_empty() {
            return ChangeResult::Error(ChangeError::DatabaseError(format!("{errors:?}")));
        }

        ChangeResult::Success
    }
}

impl IntoChangeResult for Result<SqliteQueryResult, ()> {
    fn into_change_result(self) -> ChangeResult {
        match self {
            Ok(_) => ChangeResult::Success,
            Err(()) => ChangeResult::Error(ChangeError::DefaultError),
        }
    }
}

pub trait MapToQueryError<V> {
    fn map_query_error(self) -> Result<V, QueryError>;
}

impl<V> MapToQueryError<V> for Result<V, Error> {
    fn map_query_error(self) -> Result<V, QueryError> {
        self.map_err(|_| QueryError::Default)
    }
}
