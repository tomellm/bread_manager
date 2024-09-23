pub mod profiles;
pub mod records;
pub mod possible_links;
mod utils;

use std::sync::Arc;

use pollster::FutureExt;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteQueryResult},
    SqlitePool,
};
use uuid::Uuid;

use crate::{
    model::{profiles::Profile, records::ExpenseRecord},
    utils::{
        changer::{ActionType, Response},
        communicator::{Communicator, DataContainer, GetKey},
    },
};

use self::{profiles::DbProfiles, records::DbRecords};

pub struct DB {
    profiles_container: DataContainer<Uuid, Profile, DbProfiles>,
    records_container: DataContainer<Uuid, ExpenseRecord, DbRecords>,
}

impl DB {
    pub fn get_db(drop: bool) -> Result<Self, ()> {
        let options = SqliteConnectOptions::new()
            .filename("db/bread_manager.sqlite3")
            .create_if_missing(true);

        let pool = Arc::new(
            SqlitePool::connect_with(options)
                .block_on()
                .map_err(|_| ())?,
        );
        let profiles_writer = DbProfiles { pool: pool.clone() };
        let records_writer = DbRecords { pool: pool.clone() };

        let profiles_container = DataContainer::new(profiles_writer, drop);
        let records_container = DataContainer::new(records_writer, drop);

        Ok(Self {
            profiles_container,
            records_container,
        })
    }

    pub fn profiles_signal(&mut self) -> Communicator<Uuid, Profile> {
        self.profiles_container.signal()
    }

    pub fn records_signal(&mut self) -> Communicator<Uuid, ExpenseRecord> {
        self.records_container.signal()
    }

    pub fn update(&mut self) {
        self.profiles_container.update();
        self.records_container.update();
    }
}

fn error_to_response<Key, Value>(
    query_result: Result<SqliteQueryResult, sqlx::error::Error>,
    action: &ActionType<Key, Value>,
) -> Response<Key, Value>
where
    Key: Clone + Send + Sync + 'static,
    Value: GetKey<Key> + Clone + Send + Sync + 'static,
{
    match query_result {
        Ok(_) => Response::ok(action),
        Err(err) => Response::err(action, format!("{err:?}")),
    }
}
