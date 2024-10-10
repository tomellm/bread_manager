use egui::TextEdit;

use crate::model::records::ExpenseRecord;

use super::{box_dyn, TableFilter};

#[derive(Default)]
pub struct TagsFilter(Option<String>);

impl TableFilter for TagsFilter {
    fn name(&self) -> &str {
        "Tags"
    }
    fn active(&self) -> bool {
        self.0.is_some()
    }
    fn deactivate(&mut self) {
        self.0 = None;
    }
    fn display(&mut self, ui: &mut egui::Ui) {
        if let Some(tags_filter) = &mut self.0 {
            ui.add(TextEdit::singleline(tags_filter));
        }
    }
    fn filter(&self) -> Option<super::DataFilter> {
        self.0.as_ref().map(|tags_filter| {
            let tags = tags_filter.clone();
            box_dyn(move |record: &ExpenseRecord| {
                for tag in record.tags() {
                    if tag.contains(&tags) {
                        return true;
                    }
                }
                false
            })
        })
    }
    fn filter_activation(&mut self, ui: &mut egui::Ui) {
        if ui.button("tags").clicked() {
            self.0 = Some(String::default());
        }
    }
}
