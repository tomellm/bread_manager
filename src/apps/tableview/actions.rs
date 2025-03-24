use egui::Ui;
use hermes::{
    actor::Actor, carrier::execute::ImplExecuteCarrier,
    container::data::ImplData,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use sea_query::Expr;

use crate::{
    db,
    model::records::{ExpenseRecord, ExpenseRecordState},
};

use super::DbRecord;

pub struct ActionState {
    actor: Actor,
}

impl ActionState {
    pub fn new(actor: Actor) -> Self {
        Self { actor }
    }

    pub fn display_actions(
        &mut self,
        records: &mut impl ImplData<ExpenseRecord>,
        filter: impl FnMut(&&ExpenseRecord) -> bool + Copy,
        ui: &mut Ui,
    ) {
        let uuids = || records.data().iter().filter(filter).map(|r| **r.uuid());

        ui.label("Select a action to apply to the filtered expense records.");
        ui.separator();
        if ui.button("ignore").clicked() {
            self.actor.execute(
                DbRecord::update_many()
                    .filter(db::records::Column::Uuid.is_in(uuids()))
                    .col_expr(
                        db::records::Column::State,
                        Expr::value(ExpenseRecordState::Ignored),
                    ),
            );
        }

        if ui.button("delete").clicked() {
            self.actor.execute(
                DbRecord::update_many()
                    .filter(db::records::Column::Uuid.is_in(uuids()))
                    .col_expr(
                        db::records::Column::State,
                        Expr::value(ExpenseRecordState::Deleted),
                    ),
            );
        }
    }
}
