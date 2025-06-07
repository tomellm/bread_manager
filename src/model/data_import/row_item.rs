use std::mem;

use crate::{
    db::InitUuid,
    model::transactions::{
        datetime::DatetimeUuid, movement::MovementUuid,
        properties::TransactionProperties, special_content::SpecialContentUuid,
        text_content::TextContentUuid,
    },
    uuid_impls,
};

pub type ModelImportRowItem = ImportRowItem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportRowItem {
    pub uuid: RowItemUuid,
    pub item_index: usize,
    pub content: String,
    pub parsed_content_ref: ContentRef,
}

uuid_impls!(RowItemUuid);

impl ImportRowItem {
    pub fn init(touple: (usize, String)) -> Self {
        Self {
            uuid: RowItemUuid::init(),
            item_index: touple.0,
            content: touple.1,
            parsed_content_ref: ContentRef::None,
        }
    }

    pub fn new(
        uuid: RowItemUuid,
        index: usize,
        content: String,
        parsed_content_ref: ContentRef,
    ) -> Self {
        Self {
            uuid,
            item_index: index,
            content,
            parsed_content_ref,
        }
    }

    pub fn set_movement_ref(&mut self, movement: MovementUuid) {
        let _ = mem::replace(
            &mut self.parsed_content_ref,
            ContentRef::Movement(movement),
        );
    }
    pub fn set_datetime_ref(&mut self, datetime: DatetimeUuid) {
        let _ = mem::replace(
            &mut self.parsed_content_ref,
            ContentRef::Datetime(datetime),
        );
    }
    pub fn set_text_ref(&mut self, text_content: TextContentUuid) {
        let _ = mem::replace(
            &mut self.parsed_content_ref,
            ContentRef::Text(text_content),
        );
    }
    pub fn set_special_ref(&mut self, special_content: SpecialContentUuid) {
        let _ = mem::replace(
            &mut self.parsed_content_ref,
            ContentRef::Special(special_content),
        );
    }

    pub fn set_property_ref(&mut self, prop: &TransactionProperties) {
        match prop {
            TransactionProperties::Datetime(datetime) => {
                self.set_datetime_ref(datetime.uuid)
            }
            TransactionProperties::Movement(movement) => {
                self.set_movement_ref(movement.uuid)
            }
            TransactionProperties::Text(text_content) => {
                self.set_text_ref(text_content.uuid)
            }
            TransactionProperties::Special(special_content) => {
                self.set_special_ref(special_content.uuid)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentRef {
    Datetime(DatetimeUuid),
    Movement(MovementUuid),
    Text(TextContentUuid),
    Special(SpecialContentUuid),
    None,
}
