mod amount;
mod date;
mod description;
mod origin;
mod tags;
mod uuid;

use std::sync::Arc;

use amount::AmountFilter;
use date::DateFilter;
//use description::DescriptionFilter;
use egui::Ui;
//use origin::OriginFilter;
//use tags::TagsFilter;
use uuid::UuidFilter;

use crate::model::transactions::Transaction;

pub(super) struct FilterState {
    filter: Arc<dyn Fn(&Transaction) -> bool + Send + Sync>,
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
                //DescriptionFilter::default().into(),
                //TagsFilter::default().into(),
                //OriginFilter::default().into(),
            ],
        }
    }
}

impl FilterState {
    pub(super) fn filter(&self, record: &Transaction) -> bool {
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
            self.filter =
                Arc::new(move |r| filters.iter().all(|filter| filter(r)));
        }
    }

    pub(super) fn display_filters(&mut self, ui: &mut Ui) {
        if ui.button("apply filter").clicked() {
            self.set_filter();
        }
        ui.separator();
        for filter in self.filters.iter_mut() {
            filter.ui_update(ui);
            ui.add_space(5.)
        }
    }
}

fn box_dyn(
    func: impl Fn(&Transaction) -> bool + Send + Sync + 'static,
) -> DataFilter {
    Box::new(func) as DataFilter
}

type DataFilter = Box<dyn Fn(&Transaction) -> bool + Send + Sync + 'static>;

impl<T> From<T> for Box<dyn TableFilter + Send + 'static>
where
    T: TableFilter + Send + 'static,
{
    fn from(value: T) -> Self {
        Box::new(value) as Box<dyn TableFilter + Send + 'static>
    }
}

pub trait TableFilter {
    fn ui_update(&mut self, ui: &mut Ui) {
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
