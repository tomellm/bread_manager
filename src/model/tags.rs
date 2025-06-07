use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

use crate::{db::InitUuid, uuid_impls};

pub type ModelTag = Tag;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    FromQueryResult,
    PartialOrd,
    Ord,
)]
pub struct Tag {
    pub uuid: TagUuid,
    pub tag: String,
    pub description: String,
}

impl Tag {
    pub fn init(tag: String, description: String) -> Self {
        Self::new(TagUuid::init(), tag, description)
    }
    pub fn new(uuid: TagUuid, tag: String, description: String) -> Self {
        Self {
            uuid,
            tag,
            description,
        }
    }
}

uuid_impls!(TagUuid);
