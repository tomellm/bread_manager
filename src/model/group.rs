use chrono::{DateTime, Local};

use crate::{db::InitUuid, uuid_impls};

pub type ModelGroup = Group;

pub struct Group {
    pub uuid: GroupUuid,
    pub datetime_created: DateTime<Local>,
}

uuid_impls!(GroupUuid);

impl Group {
    pub fn init() -> Self {
        Self {
            uuid: GroupUuid::init(),
            datetime_created: Local::now(),
        }
    }
}
