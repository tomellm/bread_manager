use egui::TextEdit;

use crate::model::records::ExpenseRecord;

use super::{box_dyn, TableFilter};

#[derive(Default)]
pub struct DescriptionFilter(Option<DescriptionFilterType>);

#[derive(Clone)]
enum DescriptionFilterType {
    Contains(String),
    Exact(String),
}

impl DescriptionFilterType {
    fn default_contains() -> Self {
        Self::Contains(String::default())
    }
    fn default_exact() -> Self {
        Self::Exact(String::default())
    }
}

impl TableFilter for DescriptionFilter {
    fn name(&self) -> &str {
        "Description"
    }

    fn active(&self) -> bool {
        self.0.is_some()
    }

    fn deactivate(&mut self) {
        self.0 = None;
    }

    fn display(&mut self, ui: &mut egui::Ui) {
        if let Some(desc_filter) = &mut self.0 {
            let value = match desc_filter {
                DescriptionFilterType::Contains(val) => {
                    ui.label("contains");
                    val
                }
                DescriptionFilterType::Exact(val) => {
                    ui.label("exact");
                    val
                }
            };
            ui.add(TextEdit::singleline(value));
        }
    }

    fn filter(&self) -> Option<super::DataFilter> {
        self.0
            .as_ref()
            .map(|desc_filter| match desc_filter.clone() {
                DescriptionFilterType::Contains(contains_desc) => {
                    box_dyn(move |record: &ExpenseRecord| {
                        record
                            .description_container()
                            .as_ref()
                            .map(|cont| cont.str_overlaps_with(&contains_desc))
                            .unwrap_or(false)
                    })
                }
                DescriptionFilterType::Exact(exact_desc) => {
                    box_dyn(move |record: &ExpenseRecord| {
                        record
                            .description_container()
                            .as_ref()
                            .map(|cont| {
                                cont.str_overlaps_with_exact(&exact_desc)
                            })
                            .unwrap_or(false)
                    })
                }
            })
    }

    fn filter_activation(&mut self, ui: &mut egui::Ui) {
        if ui.button("contains").clicked() {
            self.0 = Some(DescriptionFilterType::default_contains());
        }
        if ui.button("exact").clicked() {
            self.0 = Some(DescriptionFilterType::default_exact());
        }
    }
}
