mod core_linking;
mod duplicate_links;
mod transfer_links;

use core_linking::{
    are_not_considered_overlapping, calculate_probability,
    merge_to_link_identities, records_that_are_not_transfers,
};
use std::ops::Deref;
use std::sync::Arc;
use std::{fmt::Display, future::Future};
use tracing::info;

use duplicate_links::evaluate_if_duplicate_link;
use hermes::{
    carrier::{execute::ImplExecuteCarrier, query::ImplQueryCarrier},
    container::{data::ImplData, projecting::ProjectingContainer},
    factory::Factory,
    ToActiveModel,
};
use sea_orm::{EntityOrSelect, EntityTrait, QueryTrait};
use transfer_links::{
    delete_related_transfer_links_query, evaluate_if_transfer_link,
};
use uuid::Uuid;

use crate::db::link::DbLink;
use crate::db::{possible_links::DbPossibleLink, records::DbRecord};

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
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
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
    pub fn init(
        factory: Factory,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            let mut records =
                factory.builder().name("linker_records").projector();
            let mut possible_links =
                factory.builder().name("linker_possible_links").projector();
            let mut links = factory.builder().name("linker_links").projector();

            records.stored_query(DbRecord::find().select());
            possible_links.stored_query(DbPossibleLink::find().select());
            links.stored_query(DbLink::find().select());

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
        self.records.data().iter().fold(
            (None, None),
            |(mut neg, mut pos), record| {
                let uuid = record.uuid();
                if uuid.eq(&possible_link.leading) {
                    neg = Some(record);
                }
                if uuid.eq(&possible_link.following) {
                    pos = Some(record);
                }
                (neg, pos)
            },
        )
    }

    pub fn create_transfer_link(&mut self, possible_link: &PossibleLink) {
        assert!(possible_link.link_type.eq(&LinkType::Transfer));
        let delete_query = delete_related_transfer_links_query(possible_link);
        let link_to_save = Link::from(possible_link.clone());

        self.links.execute_many(move |builder| {
            builder.execute(DbLink::insert(link_to_save.dml()));
            builder.execute(delete_query);
        });
    }

    /// Find all the links with the existing records and the new list of
    /// records.
    ///
    /// Here we assume that the list of records is not contained in the
    /// existing list.
    pub fn find_links_from_new_records<'a>(
        &'a mut self,
        new_records: impl Iterator<Item = &'a ExpenseRecord>,
    ) -> Vec<PossibleLink> {
        Self::find_all_possible_links(
            new_records,
            records_that_are_not_transfers(self.records.data(), self.links.data())
                .into_iter(),
            self.links.data(),
            self.possible_links.data(),
        )
    }

    /// Find all the links between all the existing records
    pub fn find_links_in_existing_records(
        &mut self,
    ) -> impl Future<Output = ()> + Send + 'static {
        let mut actor = self.records.actor();

        let records = Arc::clone(self.records.data());
        let links = Arc::clone(self.links.data());
        let possible_links = Arc::clone(self.possible_links.data());

        async move {
            let records = records_that_are_not_transfers(&records, &links);

            let links = Self::find_all_possible_links(
                records.iter().map(Deref::deref),
                records.iter().map(Deref::deref),
                &links,
                &possible_links,
            );

            if !links.is_empty() {
                actor.execute(DbPossibleLink::insert_many(
                    links.into_iter().map(ToActiveModel::dml),
                ));
            }
        }
    }

    /// Find all the possible links between the two records, this list needs
    /// to be cleaned up later since there might be links that are between
    /// records that have been deemed duplicates
    pub fn find_all_possible_links<'a>(
        outer_records: impl Iterator<Item = &'a ExpenseRecord>,
        inner_records: impl Iterator<Item = &'a ExpenseRecord>,
        all_links: &[Link],
        all_poss_links: &[PossibleLink],
    ) -> Vec<PossibleLink> {
        let outer = outer_records.into_iter().collect::<Vec<_>>();
        let inner = inner_records.into_iter().collect::<Vec<_>>();

        // Find all of the possible links
        let mut possible_links = outer
            .iter()
            .flat_map(|outer| {
                inner.iter().flat_map(|inner| {
                    vec![
                        evaluate_if_transfer_link(outer, inner),
                        evaluate_if_duplicate_link(outer, inner),
                    ]
                })
            })
            .flatten()
            .collect::<Vec<_>>();

        let link_uuids = merge_to_link_identities(all_links, all_poss_links);

        possible_links.retain(|new_link| {
            // check that all
            link_uuids.iter().all(|identity| {
                are_not_considered_overlapping(identity, new_link)
            })
        });

        info!(
            msg = format!(
                "Tried to find possible links between [{}] total and [{}] new records. Found [{}]",
                outer.len(),
                inner.len(),
                possible_links.len()
            )
        );
        possible_links
    }

    pub fn calculate_probability(
        &self,
        falloff_steepness: f64,
        offset_days: f64,
    ) -> impl std::future::Future<Output = ()> + Send + 'static {
        calculate_probability(
            &self.possible_links,
            &self.records,
            &self.possible_links,
            falloff_steepness,
            offset_days,
        )
    }

    pub fn delete_related_links_query(
        &self,
        possible_link: &PossibleLink,
    ) -> impl QueryTrait + Send + 'static {
        delete_related_transfer_links_query(possible_link)
    }
}
