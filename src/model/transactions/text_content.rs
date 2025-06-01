use crate::{db::InitUuid, model::group::GroupUuid, uuid_impls};

use super::content_description::ContentDescription;

pub(crate) type ModelTextContent = TextContent;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextContent {
    pub uuid: TextContentUuid,
    pub content: String,
    pub description: ContentDescription,
    pub group_uuid: GroupUuid,
}

impl TextContent {
    pub fn init(
        content: String,
        description: ContentDescription,
        group_uuid: GroupUuid,
    ) -> Self {
        Self::new(TextContentUuid::init(), content, description, group_uuid)
    }

    pub fn new(
        uuid: TextContentUuid,
        content: String,
        description: ContentDescription,
        group_uuid: GroupUuid,
    ) -> Self {
        Self {
            uuid,
            content,
            description,
            group_uuid,
        }
    }
}

uuid_impls!(TextContentUuid);
