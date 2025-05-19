use egui::{TextEdit, Ui};

use crate::model::transactions::Transaction;

use super::{box_dyn, DataFilter, TableFilter};

#[derive(Default)]
pub struct UuidFilter(Option<String>);

impl From<String> for UuidFilter {
    fn from(value: String) -> Self {
        Self(Some(value))
    }
}

impl TableFilter for UuidFilter {
    fn name(&self) -> &str {
        "Uuid"
    }
    fn active(&self) -> bool {
        self.0.is_some()
    }
    fn deactivate(&mut self) {
        self.0 = None;
    }
    fn display(&mut self, ui: &mut Ui) {
        if let Some(uuid_filter) = &mut self.0 {
            ui.add(TextEdit::singleline(uuid_filter));
        }
    }
    fn filter(&self) -> Option<DataFilter> {
        self.0.as_ref().map(|uuid_filter| {
            let uuid = uuid_filter.clone();
            box_dyn(move |record: &Transaction| {
                record.uuid.to_string().eq(&uuid)
            })
        })
    }
    fn filter_activation(&mut self, ui: &mut Ui) {
        if ui.button("uuid").clicked() {
            self.0 = String::default().into();
        }
    }
}
