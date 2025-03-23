use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryTrait};

use crate::{
    db::possible_links::{Column, DbPossibleLink},
    model::records::ExpenseRecord,
};

use super::{
    core_linking::amounts_empty, LinkType, PossibleLink, PossibleLinkState,
};

pub fn evaluate_if_duplicate_link(
    left: &ExpenseRecord,
    right: &ExpenseRecord,
) -> Option<PossibleLink> {
    // if the uuids are the same
    // or the amounts are 0
    // or the amounts are not exactly the same
    if left.has_same_uuid(right)
        || amounts_empty(left, right)
        || !left.amount().eq(right.amount())
    {
        return None;
    }
    Some(PossibleLink::from_uuids(
        *left.uuid(),
        *right.uuid(),
        LinkType::DuplicateOf,
    ))
}

pub fn delete_related_duplicate_links_query(
    possible_link: &PossibleLink,
) -> impl QueryTrait + Send + 'static {
    DbPossibleLink::update_many()
        .col_expr(Column::State, PossibleLinkState::Deleted.into())
        .filter(
            Column::LinkType
                .eq(LinkType::DuplicateOf)
                .and(Column::State.eq(PossibleLinkState::Active))
                .and(Column::Leading.eq(*possible_link.following))
                .and(Column::Following.eq(*possible_link.leading)),
        )
}
