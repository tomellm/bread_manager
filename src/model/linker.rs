use std::fmt::Display;
use std::{collections::HashMap, f64::consts::E};

use hermes::carrier::execute::ImplExecuteCarrier;
use hermes::container::data::ImplData;
use hermes::factory::Factory;
use hermes::ToActiveModel;
use hermes::{carrier::query::ImplQueryCarrier, container::projecting::ProjectingContainer};
use itertools::Itertools;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryTrait};
use sea_query::Expr;
use tracing::{info, warn};
use uuid::Uuid;

use crate::db::link::DbLink;
use crate::db::possible_links::DbPossibleLink;
use crate::db::records::DbRecord;
use crate::db::{self, possible_links};

use super::records::{ExpenseRecord, ExpenseRecordUuid};

#[derive(Clone, Debug)]
pub struct Link {
    pub uuid: Uuid,
    pub leading: ExpenseRecordUuid,
    pub following: ExpenseRecordUuid,
    pub deleted: bool,
    pub link_type: LinkType,
}

/// This LinkType describes the relationship between the leading to the
/// following records linked to in the link object.
#[derive(Debug, Clone, Copy, Default)]
pub enum LinkType {
    /// Means that the leading records amount was transferred to the
    /// following records. Also means that the records only describe a
    /// movement between internal accounts
    #[default]
    Transfer,
    /// Means that the leading record is a duplicate of the following record.
    /// In this case there can also be many different records that are
    /// duplicates of the same one record
    DuplicateOf,
}

impl Display for LinkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Link {
    pub fn contains(&self, record: &ExpenseRecord) -> bool {
        self.leading.eq(record.uuid()) || self.following.eq(record.uuid())
    }
}

impl From<PossibleLink> for Link {
    fn from(
        PossibleLink {
            uuid,
            leading: negative,
            following: positive,
            link_type,
            ..
        }: PossibleLink,
    ) -> Self {
        Self {
            uuid,
            leading: negative,
            following: positive,
            deleted: false,
            link_type,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PossibleLink {
    pub uuid: Uuid,
    /// negative
    pub leading: ExpenseRecordUuid,
    /// positive
    pub following: ExpenseRecordUuid,
    pub probability: f64,
    pub state: PossibleLinkState,
    pub link_type: LinkType,
}

#[derive(Clone, Debug)]
pub enum PossibleLinkState {
    Active,
    Deleted,
    Converted,
}

impl Display for PossibleLinkState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl PossibleLink {
    pub fn from_uuids(
        left: ExpenseRecordUuid,
        right: ExpenseRecordUuid,
        link_type: LinkType,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            leading: left,
            following: right,
            probability: 1f64,
            state: PossibleLinkState::Active,
            link_type,
        }
    }

    pub fn contains(&self, record: &ExpenseRecordUuid) -> bool {
        self.following.eq(record) || self.leading.eq(record)
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        self.leading.eq(&other.leading)
            || self.following.eq(&other.leading)
            || self.leading.eq(&other.following)
            || self.following.eq(&other.following)
    }
}

pub struct Linker {
    possible_links: ProjectingContainer<PossibleLink, DbPossibleLink>,
    links: ProjectingContainer<Link, DbLink>,
    records: ProjectingContainer<ExpenseRecord, DbRecord>,
}

impl Linker {
    pub fn init(factory: Factory) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            let mut records = factory.builder().name("linker_records").projector();
            let mut possible_links = factory.builder().name("linker_possible_links").projector();
            let mut links = factory.builder().name("linker_links").projector();

            records.stored_query(DbRecord::find_all_active());
            possible_links.stored_query(DbPossibleLink::find_all_active());
            links.stored_query(DbLink::find_all_active());

            Self {
                possible_links,
                links,
                records,
            }
        }
    }

    pub fn state_update(&mut self) {
        self.possible_links.state_update(true);
        self.links.state_update(true);
        self.records.state_update(true);
    }

    pub fn get_records(
        &self,
        possible_link: &PossibleLink,
    ) -> (Option<&ExpenseRecord>, Option<&ExpenseRecord>) {
        self.records
            .data()
            .iter()
            .fold((None, None), |(mut neg, mut pos), record| {
                let uuid = record.uuid();
                if uuid.eq(&possible_link.leading) {
                    neg = Some(record);
                }
                if uuid.eq(&possible_link.following) {
                    pos = Some(record);
                }
                (neg, pos)
            })
    }

    pub fn create_link(&mut self, possible_link: &PossibleLink) {
        let delete_query = self.delete_related_links_query(possible_link);
        let link_to_save = Link::from(possible_link.clone());

        self.links.execute_many(move |builder| {
            builder.execute(DbLink::insert(link_to_save.dml()));
            builder.execute(delete_query);
        });
    }

    pub fn delete_related_links_query(
        &self,
        possible_link: &PossibleLink,
    ) -> impl QueryTrait + Send + 'static {
        DbPossibleLink::update_many()
            .col_expr(
                db::possible_links::Column::State,
                Expr::value(PossibleLinkState::Deleted.to_string()),
            )
            .filter(
                db::possible_links::Column::LinkType
                    .eq(LinkType::Transfer)
                    .and(
                        db::possible_links::Column::Leading
                            .eq(*possible_link.following)
                            .or(db::possible_links::Column::Following.eq(*possible_link.following))
                            .or(db::possible_links::Column::Leading.eq(*possible_link.leading))
                            .or(db::possible_links::Column::Following.eq(*possible_link.leading)),
                    ),
            )
    }

    pub fn calculate_probability(
        &mut self,
        falloff_steepness: f64,
        offset_days: f64,
    ) -> impl std::future::Future<Output = ()> + Send + 'static {
        let linked_records = self
            .possible_links
            .data()
            .iter()
            .flat_map(|link| vec![*link.following, *link.leading])
            .collect::<Vec<_>>();
        let records = self
            .records
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
        let links = self.possible_links.data().clone();
        let mut possible_links_actor = self.possible_links.actor();
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

                    let time_distance = (*positive.datetime() - *negative.datetime())
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
                        / (1f64 + E.powf((1f64 - falloff_steepness) * time_distance - offset_days));
                    (uuid, new_val)
                })
                .collect_vec();

            possible_links_actor.execute_many(|builder| {
                uuid_and_vals.into_iter().for_each(|(uuid, new_val)| {
                    builder.execute(
                        DbPossibleLink::update_many()
                            .col_expr(possible_links::Column::Probability, Expr::value(new_val))
                            .filter(possible_links::Column::Uuid.eq(uuid)),
                    );
                });
            });
        }
    }

    pub fn find_links<'a>(
        &'a mut self,
        new_records: impl Iterator<Item = &'a ExpenseRecord> + Clone,
    ) -> Vec<PossibleLink> {
        let mut possible_links = vec![];

        let all_records = self
            .records
            .data()
            .iter()
            // remove all records that are already part of a link
            .filter(|record| !self.links.data().iter().any(|link| link.contains(record)));

        // find all the links with the remaining records
        let links_to_existing_records =
            self.find_all_possible_links(new_records.clone(), all_records);

        possible_links.extend(links_to_existing_records);

        info!(
            msg = format!(
                "Tried to find possible links between [{}] total and [{}] new records. Found [{}]",
                self.records.data().len(),
                new_records.count(),
                possible_links.len()
            )
        );
        possible_links
    }

    fn amounts_empty(&self, existing: &ExpenseRecord, new: &ExpenseRecord) -> bool {
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

    fn amounts_are_opposites(&self, left: &ExpenseRecord, right: &ExpenseRecord) -> bool {
        (left.amount() * -1).eq(right.amount())
    }

    fn create_transfer_possible_link(
        &self,
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

    fn evaluate_if_link(
        &self,
        left: &ExpenseRecord,
        right: &ExpenseRecord,
    ) -> Option<PossibleLink> {
        // if the uuids are the same
        if left.uuid().eq(right.uuid())
            // or the amounts are 0
            || self.amounts_empty(left, right)
            // or the amounts are not opposites
            || !self.amounts_are_opposites(left, right)
        {
            // no match
            return None;
        }
        // otherwise create the link
        Some(self.create_transfer_possible_link(left, right))
    }

    fn find_all_possible_links<'a>(
        &self,
        outer_records: impl Iterator<Item = &'a ExpenseRecord> + Clone,
        inner_records: impl Iterator<Item = &'a ExpenseRecord> + Clone,
    ) -> Vec<PossibleLink> {
        // for every outer record
        outer_records
            .into_iter()
            .flat_map(|outer| {
                // eval if there is a link with the inner records
                inner_records
                    .clone()
                    .filter_map(|inner| self.evaluate_if_link(outer, inner))
            })
            .collect::<Vec<_>>()
    }
}
