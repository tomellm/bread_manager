use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryTrait};
use sea_query::Expr;

use crate::{
    db::{
        self,
        possible_links::{Column, DbPossibleLink},
    },
    model::records::{ExpenseRecord, ExpenseRecordUuid},
};

use super::{
    core_linking::amounts_empty, LinkType, PossibleLink, PossibleLinkState,
};

pub fn evaluate_if_transfer_link(
    left: &ExpenseRecord,
    right: &ExpenseRecord,
) -> Option<PossibleLink> {
    // if the uuids are the same
    if left.has_same_uuid(right)
        // or the amounts are 0
        || amounts_empty(left, right)
        // or the amounts are not opposites
        || !amounts_are_opposites(left, right)
    {
        // no match
        return None;
    }
    // otherwise create the link
    Some(create_transfer_possible_link(left, right))
}

fn amounts_are_opposites(left: &ExpenseRecord, right: &ExpenseRecord) -> bool {
    (left.amount() * -1).eq(right.amount())
}

fn create_transfer_possible_link(
    left: &ExpenseRecord,
    right: &ExpenseRecord,
) -> PossibleLink {
    let negative: ExpenseRecordUuid;
    let positive: ExpenseRecordUuid;
    if left.amount().is_negative() {
        negative = *left.uuid();
        positive = *right.uuid();
    } else {
        negative = *right.uuid();
        positive = *left.uuid();
    }
    PossibleLink::from_uuids(negative, positive, LinkType::Transfer)
}

pub fn delete_related_transfer_links_query(
    possible_link: &PossibleLink,
) -> impl QueryTrait + Send + 'static {
    DbPossibleLink::update_many()
        .col_expr(Column::State, PossibleLinkState::Deleted.into())
        .filter(
            Column::LinkType
                .eq(LinkType::Transfer)
                .and(Column::State.eq(PossibleLinkState::Active))
                .and(
                    Column::Leading
                        .eq(*possible_link.following)
                        .or(Column::Following.eq(*possible_link.following))
                        .or(Column::Leading.eq(*possible_link.leading))
                        .or(Column::Following.eq(*possible_link.leading)),
                ),
        )
}
