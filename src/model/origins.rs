use sea_orm::FromQueryResult;

use crate::{db::InitUuid, uuid_impls};

pub type ModelOrigin = Origin;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, FromQueryResult)]
pub struct Origin {
    pub uuid: OriginUuid,
    pub name: String,
    pub description: String,
}

impl Origin {
    pub fn init(name: String, description: String) -> Self {
        Self::new(OriginUuid::init(), name, description)
    }

    pub fn new(uuid: OriginUuid, name: String, description: String) -> Self {
        Self {
            uuid,
            name,
            description,
        }
    }
}

uuid_impls!(OriginUuid);
