use std::{collections::HashMap, f64::consts::E};

use egui::ahash::{HashSet, HashSetExt};
use hermes::{carrier::execute::ImplExecuteCarrier, container::data::ImplData};
use itertools::Itertools;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use sea_query::Expr;
use tracing::warn;

use crate::{
    db::possible_links::{self, DbPossibleLink},
    model::records::{ExpenseRecord, ExpenseRecordUuid},
};

use super::{Link, LinkType, PossibleLink};

pub fn amounts_empty(existing: &ExpenseRecord, new: &ExpenseRecord) -> bool {
    if *existing.amount() == 0 {
        warn!(
            "The amount of the existing record [{:?}] is 0.",
            existing.uuid()
        );
        true
    } else if *new.amount() == 0 {
        warn!("The amount of the new record [{:?}] is 0.", new.uuid());
        true
    } else {
        false
    }
}

pub fn records_that_are_not_transfers<'a>(
    records: &'a impl ImplData<ExpenseRecord>,
    links: &'a impl ImplData<Link>,
) -> Vec<&'a ExpenseRecord> {
    let transfer_links = links
        .data()
        .iter()
        .filter(|link| link.link_type == LinkType::Transfer);

    records
        .data()
        .iter()
        // remove all records that are already part of a link
        .filter(|record| {
            !transfer_links.clone().any(|link| link.contains(record))
        })
        .collect_vec()
}

pub fn calculate_probability(
    possible_links: &impl ImplData<PossibleLink>,
    records: &impl ImplData<ExpenseRecord>,
    exec_carr: &impl ImplExecuteCarrier,
    falloff_steepness: f64,
    offset_days: f64,
) -> impl std::future::Future<Output = ()> + Send + 'static {
    let linked_records = possible_links
        .data()
        .iter()
        .flat_map(|link| vec![*link.following, *link.leading])
        .collect::<Vec<_>>();
    let records = records
        .data()
        .iter()
        .filter_map(|val| {
            if linked_records.contains(val.uuid()) {
                Some((**val.uuid(), val.clone()))
            } else {
                None
            }
        })
        .collect::<HashMap<_, _>>();
    let links = possible_links.data().clone();
    let mut actor = exec_carr.actor();
    async move {
        let probs = links
            .iter()
            .map(|link| {
                let Some(positive) = records.get(&link.following) else {
                    return (link.uuid, f64::INFINITY);
                };
                let Some(negative) = records.get(&link.leading) else {
                    return (link.uuid, f64::INFINITY);
                };

                let time_distance = (*positive.datetime()
                    - *negative.datetime())
                .num_days()
                .abs() as f64;
                (link.uuid, time_distance)
            })
            .collect::<HashMap<_, _>>();

        let uuid_and_vals = links
            .into_iter()
            .map(|link| {
                let uuid = link.uuid;
                let time_distance = probs.get(&link.uuid).unwrap();
                let new_val = 1f64
                    / (1f64
                        + E.powf(
                            (1f64 - falloff_steepness) * time_distance
                                - offset_days,
                        ));
                (uuid, new_val)
            })
            .collect_vec();

        actor.execute_many(|builder| {
            uuid_and_vals.into_iter().for_each(|(uuid, new_val)| {
                builder.execute(
                    DbPossibleLink::update_many()
                        .col_expr(
                            possible_links::Column::Probability,
                            Expr::value(new_val),
                        )
                        .filter(possible_links::Column::Uuid.eq(uuid)),
                );
            });
        });
    }
}

pub fn merge_to_link_identities(
    all_links: &impl ImplData<Link>,
    all_poss_links: &impl ImplData<PossibleLink>
) -> HashSet<LinkIdentity> { 
    let link_uuids = all_links.data().iter().fold(
        HashSet::new(),
        |mut acc, link| {
            acc.insert(link.into());
            acc
        },
    );
    all_poss_links
        .data()
        .iter()
        .fold(link_uuids, |mut acc, link| {
            acc.insert(link.into());
            acc
        })
}

/// Clean up the links depending on the exising links
/// - find all exact matches in the possible_links
/// - find all exact matches in the links
/// - for transfers find all possible links where one of the sides
///   is the same
pub fn are_not_considered_overlapping(
    LinkIdentity {
        leading,
        following,
        link_type,
        variant,
    }: &LinkIdentity,
    new_link: &PossibleLink,
) -> bool {
    let same_leading = leading.eq(&new_link.leading);
    let same_following = following.eq(&new_link.following);
    let same_link_type = link_type.eq(&new_link.link_type);

    // is completly same link
    let full_match = same_leading && same_following && same_link_type;

    // a link can only be part of one transfer but only actual
    // links count as confirmed transfers
    let part_of_transfer = match same_link_type
        && link_type.eq(&LinkType::Transfer)
        && variant.eq(&LinkVariant::Link)
    {
        true => return same_leading || same_following,
        false => false,
    };

    // if a is dupe of b then the link with type duplicate
    // from b -> a is also to ignore
    let inverse_duplicate = match same_link_type
        && link_type.eq(&LinkType::DuplicateOf)
        && variant.eq(&LinkVariant::Link)
    {
        true => leading.eq(&new_link.following),
        false => following.eq(&new_link.leading),
    };

    // is not a full match or part of a transfer or not the
    // inverse of duplicate link means its a valid link
    !(full_match || part_of_transfer || inverse_duplicate)
}

#[derive(Eq, Hash, PartialEq)]
pub(super) struct LinkIdentity {
    leading: ExpenseRecordUuid,
    following: ExpenseRecordUuid,
    link_type: LinkType,
    variant: LinkVariant,
}

impl From<&PossibleLink> for LinkIdentity {
    fn from(value: &PossibleLink) -> Self {
        Self {
            leading: value.leading,
            following: value.following,
            link_type: value.link_type,
            variant: LinkVariant::PossibleLink,
        }
    }
}

impl From<&Link> for LinkIdentity {
    fn from(value: &Link) -> Self {
        Self {
            leading: value.leading,
            following: value.following,
            link_type: value.link_type,
            variant: LinkVariant::Link,
        }
    }
}

#[derive(Eq, Hash, PartialEq)]
pub(super) enum LinkVariant {
    Link,
    PossibleLink,
}
