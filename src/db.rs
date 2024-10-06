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
use possible_links::DbPossibleLinks;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteQueryResult},
    Error, SqlitePool,
};
use uuid::Uuid;

use crate::model::{linker::PossibleLink, profiles::Profile, records::ExpenseRecord};

use self::{profiles::DbProfiles, records::DbRecords};

pub struct DB {
    profiles_container: DataContainer<Uuid, Profile, DbProfiles>,
    records_container: DataContainer<Uuid, ExpenseRecord, DbRecords>,
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
        let possible_links_container = DataContainer::new((&pool, drop).into()).await;
        Ok(Self {
            profiles_container,
            records_container,
            possible_links_container,
        })
    }

    pub fn profiles_signal(&mut self) -> Communicator<Uuid, Profile> {
        self.profiles_container.communicator()
    }

    pub fn records_signal(&mut self) -> Communicator<Uuid, ExpenseRecord> {
        self.records_container.communicator()
    }

    pub fn possible_links_signal(&mut self) -> Communicator<Uuid, PossibleLink> {
        self.possible_links_container.communicator()
    }

    pub fn state_update(&mut self) {
        self.profiles_container.state_update();
        self.records_container.state_update();
        self.possible_links_container.state_update();
    }
}

pub trait IntoChangeResult {
    fn into_change_result(self) -> ChangeResult;
}

impl<R> IntoChangeResult for Result<SqliteQueryResult, R> {
    fn into_change_result(self) -> ChangeResult {
        match self {
            Ok(_) => ChangeResult::Success,
            Err(_) => ChangeResult::Error(ChangeError::DefaultError),
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
