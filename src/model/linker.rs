use std::{collections::HashMap, f64::consts::E};

use diesel::associations::HasTable;
use diesel::{Insertable, QueryDsl, RunQueryDsl, SelectableHelper};
use hermes::container::projecting::ProjectingContainer;
use hermes::factory::Factory;
use tracing::{info, warn};
use uuid::Uuid;

use crate::db::link::DbLink;
use crate::db::possible_links::DbPossibleLink;
use crate::schema::expense_records::dsl::expense_records as records_table;
use crate::schema::links::dsl::links as links_table;
use crate::schema::possible_links::dsl::possible_links as possible_links_table;
use crate::{
    apps::DbConn,
    db::{
        link::LINK_FROM_DB_FN,
        possible_links::POSSIBLE_LINK_FROM_DB_FN,
        records::{DbRecord, RECORDS_FROM_DB_FN},
    },
};

use super::records::{ExpenseRecord, ExpenseRecordUuid};

#[derive(Clone, Debug)]
pub struct Link {
    pub uuid: Uuid,
    pub negative: ExpenseRecordUuid,
    pub positive: ExpenseRecordUuid,
}

impl Link {
    pub fn contains(&self, record: &ExpenseRecord) -> bool {
        self.negative.eq(record.uuid()) || self.positive.eq(record.uuid())
    }
}

impl From<PossibleLink> for Link {
    fn from(
        PossibleLink {
            uuid,
            negative,
            positive,
            ..
        }: PossibleLink,
    ) -> Self {
        Self {
            uuid,
            negative,
            positive,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PossibleLink {
    pub uuid: Uuid,
    pub negative: ExpenseRecordUuid,
    pub positive: ExpenseRecordUuid,
    pub probability: f64,
}

impl PossibleLink {
    pub fn from_uuids(negative: ExpenseRecordUuid, positive: ExpenseRecordUuid) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            negative,
            positive,
            probability: 1f64,
        }
    }

    pub fn contains(&self, record: &ExpenseRecordUuid) -> bool {
        self.positive.eq(record) || self.negative.eq(record)
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        self.negative.eq(&other.negative)
            || self.positive.eq(&other.negative)
            || self.negative.eq(&other.positive)
            || self.positive.eq(&other.positive)
    }
}

pub struct Linker {
    possible_links: ProjectingContainer<PossibleLink, DbPossibleLink, DbConn>,
    links: ProjectingContainer<Link, DbLink, DbConn>,
    records: ProjectingContainer<ExpenseRecord, DbRecord, DbConn>,
}

impl Linker {
    pub fn init(
        factory: Factory<DbConn>,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            let mut records = factory.builder().projector_arc(RECORDS_FROM_DB_FN.clone());
            let mut possible_links = factory
                .builder()
                .projector_arc(POSSIBLE_LINK_FROM_DB_FN.clone());
            let mut links = factory.builder().projector_arc(LINK_FROM_DB_FN.clone());

            let _ = records.query(|| records_table.select(DbRecord::as_select()));
            let _ =
                possible_links.query(|| possible_links_table.select(DbPossibleLink::as_select()));
            let _ = links.query(|| links_table.select(DbLink::as_select()));

            Self {
                possible_links,
                links,
                records,
            }
        }
    }

    pub fn state_update(&mut self) {
        self.possible_links.state_update();
        self.links.state_update();
        self.records.state_update();
    }

    pub fn get_records(
        &self,
        possible_link: &PossibleLink,
    ) -> (Option<&ExpenseRecord>, Option<&ExpenseRecord>) {
        (
            self.records.data.map().get(&possible_link.negative),
            self.records.data.map().get(&possible_link.positive),
        )
    }

    pub fn create_link(
        &mut self,
        possible_link: &PossibleLink,
    ) -> impl std::future::Future<Output = ()> + Send + 'static {
        let links_to_delete = self
            .possible_links
            .values()
            .iter()
            .filter_map(|other_possible_link| {
                if possible_link.overlaps(other_possible_link) {
                    Some(other_possible_link.uuid)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let delete_future = self.possible_links.delete_many(links_to_delete);
        let create_future = self.links.insert(possible_link.clone().into());
        async move {
            let _ = create_future.await;
            let _ = delete_future.await;
        }
    }

    pub fn calculate_probability(
        &mut self,
        falloff_steepness: f64,
        offset_days: f64,
    ) -> impl std::future::Future<Output = ()> + Send + 'static {
        let linked_records = self
            .possible_links
            .values()
            .iter()
            .flat_map(|link| vec![*link.positive, *link.negative])
            .collect::<Vec<_>>();
        let records = self
            .records
            .values()
            .iter()
            .filter_map(|val| {
                if linked_records.contains(&val.uuid()) {
                    Some((**val.uuid(), val.clone()))
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>();
        let links = self.possible_links.values().clone();
        let possible_links_actor = self.possible_links.actor();
        async move {
            let probs = links
                .iter()
                .map(|link| {
                    let Some(positive) = records.get(&link.positive) else {
                        return (link.uuid, f64::INFINITY);
                    };
                    let Some(negative) = records.get(&link.negative) else {
                        return (link.uuid, f64::INFINITY);
                    };

                    let time_distance = (*positive.datetime() - *negative.datetime())
                        .num_days()
                        .abs() as f64;
                    (link.uuid, time_distance)
                })
                .collect::<HashMap<_, _>>();

            let links = links
                .into_iter()
                .map(|mut link| {
                    let time_distance = probs.get(&link.uuid).unwrap();
                    link.probability = 1f64
                        / (1f64 + E.powf((1f64 - falloff_steepness) * time_distance - offset_days));
                    DbPossibleLink::from(&link)
                })
                .collect::<Vec<_>>();
            let query = diesel::insert_into(possible_links_table).values(&links);
            possible_links_actor.execute(|| query);
        }
    }

    pub fn find_links(&mut self, new_records: &[ExpenseRecord]) -> Vec<PossibleLink> {
        let mut possible_links = vec![];

        let all_records = self
            .records
            .data
            .iter()
            .filter(|record| !self.links.data.iter().any(|link| link.contains(record)));

        let internal_links = self.find_all_possible_links(new_records.iter(), new_records.iter());
        let links_to_existing_records =
            self.find_all_possible_links(new_records.iter(), all_records);

        possible_links.extend(internal_links);
        possible_links.extend(links_to_existing_records);

        info!(
            msg = format!(
                "Tried to find possible links between [{}] total and [{}] new records. Found [{}]",
                self.records.data.len(),
                new_records.len(),
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
        left.amount() * -1 == *right.amount()
    }

    fn create_possible_link(&self, left: &ExpenseRecord, right: &ExpenseRecord) -> PossibleLink {
        let negative: Uuid;
        let positive: Uuid;
        if left.amount().is_negative() {
            negative = **left.uuid();
            positive = **right.uuid();
        } else {
            negative = **right.uuid();
            positive = **left.uuid();
        }
        PossibleLink::from_uuids(negative.into(), positive.into())
    }

    fn evaluate_if_link(
        &self,
        left: &ExpenseRecord,
        right: &ExpenseRecord,
    ) -> Option<PossibleLink> {
        if left.uuid().eq(right.uuid())
            || self.amounts_empty(left, right)
            || !self.amounts_are_opposites(left, right)
        {
            return None;
        }
        Some(self.create_possible_link(left, right))
    }

    fn find_all_possible_links<'a>(
        &self,
        outer_records: impl Iterator<Item = &'a ExpenseRecord> + Clone,
        inner_records: impl Iterator<Item = &'a ExpenseRecord> + Clone,
    ) -> Vec<PossibleLink> {
        outer_records
            .into_iter()
            .flat_map(|outer| {
                inner_records
                    .clone()
                    .filter_map(|inner| self.evaluate_if_link(outer, inner))
            })
            .collect::<Vec<_>>()
    }
}
