use tracing::warn;
use uuid::Uuid;

use super::records::{ExpenseRecord, ExpenseRecordUuid};

#[derive(Clone, Debug)]
pub struct PossibleLink {
    pub uuid: Uuid,
    pub negative: ExpenseRecordUuid,
    pub positive: ExpenseRecordUuid,
    pub probability: f64
}

impl PossibleLink {
    pub fn from_uuids(
        negative: ExpenseRecordUuid,
        positive: ExpenseRecordUuid,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            negative,
            positive,
            probability: 1f64
        }
    }
}

pub struct Linker;

impl Linker {
    pub fn find_links(
        new_records: &Vec<ExpenseRecord>,
        all_records: Vec<&ExpenseRecord>,
    ) -> Vec<PossibleLink> {
        let mut possible_links = vec![];
        for existing_record in all_records {
            for new_record in new_records {
                if *existing_record.amount() == 0 {
                    warn!("The amount of the existing record [{:?}] is 0.", existing_record.uuid());
                } else if *new_record.amount() == 0 {
                    warn!("The amount of the new record [{:?}] is 0.", new_record.uuid());
                } else if existing_record.amount() * -1 == *new_record.amount() {
                    let negative: Uuid;
                    let positive: Uuid;
                    if existing_record.amount().is_negative() {
                        negative = **existing_record.uuid();
                        positive = **new_record.uuid();
                    } else {
                        negative = **new_record.uuid();
                        positive = **existing_record.uuid();
                    }
                    possible_links.push(PossibleLink::from_uuids(
                        negative.into(),
                        positive.into(),
                    ))
                }
            }
        }
        possible_links
    }
}
