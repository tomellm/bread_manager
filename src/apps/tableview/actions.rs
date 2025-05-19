use egui::Ui;
use hermes::{actor::Actor, container::data::ImplData};

use crate::model::transactions::Transaction;

pub struct ActionState {
    actor: Actor,
}

impl ActionState {
    pub fn new(actor: Actor) -> Self {
        Self { actor }
    }

    pub fn display_actions(
        &mut self,
        transacts: &mut impl ImplData<Transaction>,
        filter: impl FnMut(&&Transaction) -> bool + Copy,
        ui: &mut Ui,
    ) {
        let uuids = || transacts.data().iter().filter(filter).map(|t| t.uuid);

        ui.label("Select a action to apply to the filtered expense records.");
        ui.separator();
        if ui.button("ignore").clicked() {
            //self.actor.execute(
            //    DbRecord::update_many()
            //        .filter(db::records::Column::Uuid.is_in(uuids()))
            //        .col_expr(
            //            db::records::Column::State,
            //            Expr::value(ExpenseRecordState::Ignored),
            //        ),
            //);
        }

        if ui.button("delete").clicked() {
            //self.actor.execute(
            //    DbRecord::update_many()
            //        .filter(db::records::Column::Uuid.is_in(uuids()))
            //        .col_expr(
            //            db::records::Column::State,
            //            Expr::value(ExpenseRecordState::Deleted),
            //        ),
            //);
        }
    }
}
