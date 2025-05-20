pub(crate) mod transaction_datetime_query;
pub(crate) mod transaction_movement_query;
pub(crate) mod transaction_properties;
pub(crate) mod transaction_text_query;
pub mod transaction_special_query;

use hermes::{
    carrier::{manual_query::ImplManualQueryCarrier, query::ExecutedQuery},
    container::manual,
    ContainsTables, TablesCollector,
};
use itertools::Itertools;
use sea_orm::{DatabaseConnection, DbErr, EntityOrSelect, EntityTrait};
use transaction_datetime_query::all_datetimes;
use transaction_movement_query::all_movements;
use transaction_properties::TransactionEntityContainer;

use crate::model::transactions::ModelTransaction;

use super::super::{
    builders::transaction_builder::{ToTransacHashMap, TransactionBuilder},
    entities::prelude::*,
};

pub trait TransactionQuery {
    fn all(&mut self);

    fn insert_queries(
        &mut self,
        transact: ModelTransaction,
    ) -> TransactionEntityContainer {
        let mut container = TransactionEntityContainer::default();
        container.add_transaction(transact);
        container
    }

    fn insert_many_queries(
        &mut self,
        transacts: Vec<ModelTransaction>,
    ) -> TransactionEntityContainer {
        transacts.into_iter().fold(
            TransactionEntityContainer::default(),
            |mut container, transac| {
                container.add_transaction(transac);
                container
            },
        )
    }

    fn insert(&mut self, transact: ModelTransaction);
    fn insert_many(&mut self, transacts: Vec<ModelTransaction>);
}
impl TransactionQuery for manual::Container<ModelTransaction> {
    fn all(&mut self) {
        self.manual_query(|db, mut collector| async move {
            let transactions = all_transactions(&db, &mut collector).await;
            ExecutedQuery::new_collector(collector, transactions)
        });
    }

    fn insert(&mut self, transact: ModelTransaction) {
        self.insert_queries(transact).insert_everything(self);
    }

    fn insert_many(&mut self, transacts: Vec<ModelTransaction>) {
        self.insert_many_queries(transacts).insert_everything(self);
    }
}

pub(super) async fn all_transactions(
    db: &DatabaseConnection,
    collector: &mut TablesCollector,
) -> Result<Vec<ModelTransaction>, DbErr> {
    let transactions = Transaction::find()
        .select()
        .and_find_tables(collector)
        .all(db)
        .await?
        .into_iter()
        .map(TransactionBuilder::new);

    let mut transaction_datetimes =
        all_datetimes(db, collector).await?.to_hashmap();

    let mut transaction_movements =
        all_movements(db, collector).await?.to_hashmap();

    Ok(transactions
        .map(|builder| builder.feed_datetimes(&mut transaction_datetimes))
        .map(|builder| builder.feed_movements(&mut transaction_movements))
        .map(TransactionBuilder::build)
        .collect_vec())
}
