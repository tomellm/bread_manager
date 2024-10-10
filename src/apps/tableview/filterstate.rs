mod amount;
mod date;
mod uuid;
mod tags;
mod origin;

use std::sync::Arc;

use amount::AmountFilter;
use date::DateFilter;
use egui::Ui;
use origin::OriginFilter;
use tags::TagsFilter;
use uuid::UuidFilter;

use crate::model::records::ExpenseRecord;

pub(super) struct FilterState {
    filter: Arc<dyn Fn(&ExpenseRecord) -> bool + Send + Sync>,
    filters: Vec<Box<dyn TableFilter + Send + 'static>>,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            filter: Arc::new(|_| true),
            filters: vec![
                UuidFilter::default().into(),
                AmountFilter::default().into(),
                DateFilter::default().into(),
                TagsFilter::default().into(),
                OriginFilter::default().into(),
            ],
        }
    }
}

impl FilterState {
    pub(super) fn filter(&self, record: &ExpenseRecord) -> bool {
        (self.filter)(record)
    }

    fn set_filter(&mut self) {
        let filters = self
            .filters
            .iter()
            .filter_map(|filter| filter.filter())
            .collect::<Vec<_>>();

        if filters.is_empty() {
            self.filter = Arc::new(|_| true);
        } else {
            self.filter = Arc::new(move |r| filters.iter().all(|filter| filter(r)));
        }
    }

    pub(super) fn display_filters(&mut self, ui: &mut Ui) {
        if ui.button("apply filter").clicked() {
            self.set_filter();
        }
        ui.separator();
        for filter in self.filters.iter_mut() {
            filter.ui(ui);
            ui.add_space(5.)
        }
    }
}

fn box_dyn(func: impl Fn(&ExpenseRecord) -> bool + Send + Sync + 'static) -> DataFilter {
    Box::new(func) as DataFilter
}

type DataFilter = Box<dyn Fn(&ExpenseRecord) -> bool + Send + Sync + 'static>;

impl<T> From<T> for Box<dyn TableFilter + Send + 'static>
where
    T: TableFilter + Send + 'static,
{
    fn from(value: T) -> Self {
        Box::new(value) as Box<dyn TableFilter + Send + 'static>
    }
}

pub trait TableFilter {
    fn ui(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(self.name());
            ui.horizontal(|ui| {
                if self.active() {
                    self.display(ui);
                    if ui.button("clear").clicked() {
                        self.deactivate();
                    }
                } else {
                    self.filter_activation(ui);
                }
            });
        });
    }
    fn name(&self) -> &str;
    fn active(&self) -> bool;
    fn deactivate(&mut self);
    fn display(&mut self, ui: &mut Ui);
    fn filter(&self) -> Option<DataFilter>;
    fn filter_activation(&mut self, ui: &mut Ui);
}