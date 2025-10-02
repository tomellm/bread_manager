use egui::Ui;

use crate::model::transactions::Transaction;

#[derive(Debug)]
pub struct CodeExecError {
    trxn: Transaction,
    err: String,
}

impl CodeExecError {
    pub fn new(trxn: &Transaction, err: mlua::Error) -> Self {
        Self {
            trxn: trxn.clone(),
            err: err.to_string(),
        }
    }

    pub fn as_row(&self, ui: &mut Ui) {
        ui.label(self.err.to_string());
        ui.label(self.trxn.uuid.to_string());
        ui.end_row();
    }
}
