use egui::TextEdit;

use crate::model::records::ExpenseRecord;

use super::{box_dyn, TableFilter};



#[derive(Default)]
pub struct OriginFilter(Option<String>);

impl TableFilter for OriginFilter {
    fn name(&self) -> &str {
        "Origin"
    }
    fn active(&self) -> bool {
        self.0.is_some()
    }
    fn deactivate(&mut self) {
        self.0 = None;
    }
    fn display(&mut self, ui: &mut egui::Ui) {
        if let Some(origin_filter) = &mut self.0 {
            ui.add(TextEdit::singleline(origin_filter));
        }
    }
    fn filter(&self) -> Option<super::DataFilter> {
        self.0.as_ref().map(|origin_filter| {
            let origin = origin_filter.clone();
            box_dyn(move |record: &ExpenseRecord| record.origin().contains(&origin))
        })
    }
    fn filter_activation(&mut self, ui: &mut egui::Ui) {
        if ui.button("origin").clicked() {
            self.0 = Some(String::default());
        }
    }

}

