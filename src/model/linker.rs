use data_communicator::buffered::{communicator::Communicator, query::QueryType};
use tracing::{info, warn};
use uuid::Uuid;

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
}

pub struct Linker {
    links: Communicator<Uuid, Link>,
    records: Communicator<Uuid, ExpenseRecord>,
}

impl Linker {
    pub fn init(
        links: Communicator<Uuid, Link>,
        records: Communicator<Uuid, ExpenseRecord>,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            let _ = links.query(QueryType::All).await;
            let _ = records.query(QueryType::All).await;

            Self { links, records }
        }
    }

    pub fn state_update(&mut self) {
        self.links.state_update();
        self.records.state_update();
    }

    pub fn find_links(&mut self, new_records: &[ExpenseRecord]) -> Vec<PossibleLink> {
        let mut possible_links = vec![];

        let all_records = self
            .records
            .data.iter()
            .filter(|record| !self.links.data.iter().any(|link| link.contains(record)));

        let internal_links =
            self.find_all_possible_links(new_records.iter(), new_records.iter());
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
