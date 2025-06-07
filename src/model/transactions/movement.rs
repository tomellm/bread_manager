use crate::{db::InitUuid, model::group::GroupUuid, uuid_impls};

use super::properties::OriginType;

pub type ModelMovement = Movement;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Movement {
    pub uuid: MovementUuid,
    pub origin_type: OriginType,
    pub amount: i32,
    pub group_uuid: GroupUuid,
}

uuid_impls!(MovementUuid);

impl Movement {
    pub fn init(amount: i32, group_uuid: GroupUuid) -> Self {
        Self {
            uuid: MovementUuid::init(),
            origin_type: OriginType::CsvImport,
            amount,
            group_uuid,
        }
    }

    pub fn new(
        uuid: MovementUuid,
        origin_type: OriginType,
        amount: i32,
        group_uuid: GroupUuid,
    ) -> Self {
        Self {
            uuid,
            origin_type,
            amount,
            group_uuid,
        }
    }
}
