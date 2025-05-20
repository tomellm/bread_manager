use crate::{db::InitUuid, uuid_impls};

use super::{content_description::ContentDescription, group::GroupUuid};

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
        description: String,
        group_uuid: GroupUuid,
    ) -> Self {
        let desc = ContentDescription::init(description);
        Self::new(TextContentUuid::init(), content, desc, group_uuid)
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
