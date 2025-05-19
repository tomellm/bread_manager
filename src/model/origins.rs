use uuid::Uuid;

use crate::uuid_impls;

pub type ModelOrigin = Origin;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Origin {
    pub uuid: OriginUuid,
    pub name: String,
    pub description: String,
}

impl Origin {
    pub fn new(uuid: OriginUuid, name: String, description: String) -> Self {
        Self {
            uuid,
            name,
            description,
        }
    }
}

uuid_impls!(OriginUuid);
