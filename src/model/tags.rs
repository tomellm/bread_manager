use std::ops::Deref;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::uuid_impls;

pub type ModelTag = Tag;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Tag {
    pub uuid: TagUuid,
    pub tag: String,
    pub description: String,
}

impl Tag {
    pub fn new(uuid: TagUuid, tag: String, description: String) -> Self {
        Self {
            uuid,
            tag,
            description,
        }
    }
}

uuid_impls!(TagUuid);
