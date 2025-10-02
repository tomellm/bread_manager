use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::{db::InitUuid, uuid_impls};

pub(crate) type ModelContentDescription = ContentDescription;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentDescription {
    pub uuid: ContentDescriptionUuid,
    pub description: String,
    pub datetime_created: DateTime<Local>,
}

impl ContentDescription {
    pub fn init(description: String) -> Self {
        Self::new(ContentDescriptionUuid::init(), description, Local::now())
    }
    pub fn new(
        uuid: ContentDescriptionUuid,
        description: String,
        datetime_created: DateTime<Local>,
    ) -> Self {
        Self {
            uuid,
            description,
            datetime_created,
        }
    }
}

uuid_impls!(ContentDescriptionUuid);
