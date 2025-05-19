use chrono::NaiveDate;
use egui::Ui;
use egui_extras::DatePickerButton;

use crate::model::transactions::Transaction;

use super::{box_dyn, DataFilter, TableFilter};

#[derive(Default)]
pub struct DateFilter(Option<DateFilterType>);

#[derive(Clone)]
enum DateFilterType {
    Precise(NaiveDate),
    Between(NaiveDate, NaiveDate),
}

impl DateFilterType {
    fn default_precise() -> Self {
        Self::Precise(NaiveDate::default())
    }
    fn default_between() -> Self {
        Self::Between(NaiveDate::default(), NaiveDate::default())
    }
}

impl TableFilter for DateFilter {
    fn name(&self) -> &str {
        "Date"
    }
    fn active(&self) -> bool {
        self.0.is_some()
    }
    fn deactivate(&mut self) {
        self.0 = None;
    }
    fn display(&mut self, ui: &mut Ui) {
        if let Some(date_filter) = &mut self.0 {
            match date_filter {
                DateFilterType::Precise(date) => {
                    ui.add(
                        DatePickerButton::new(date)
                            .id_salt("date_filter_precise"),
                    );
                }
                DateFilterType::Between(lower, upper) => {
                    ui.label("lower");
                    ui.add(
                        DatePickerButton::new(lower)
                            .id_salt("date_filter_between_lower"),
                    );
                    ui.label("upper");
                    ui.add(
                        DatePickerButton::new(upper)
                            .id_salt("date_filter_between_upper"),
                    );
                }
            }
        }
    }
    fn filter(&self) -> Option<DataFilter> {
        self.0
            .as_ref()
            .map(|date_filter| match date_filter.clone() {
                DateFilterType::Precise(date) => {
                    box_dyn(move |record: &Transaction| {
                        record.datetime.date.eq(&date)
                    })
                }
                DateFilterType::Between(lower, upper) => {
                    box_dyn(move |record: &Transaction| {
                        record.datetime.date >= lower
                            && record.datetime.date <= upper
                    })
                }
            })
    }
    fn filter_activation(&mut self, ui: &mut Ui) {
        if ui.button("precise").clicked() {
            self.0 = Some(DateFilterType::default_precise());
        }
        if ui.button("between").clicked() {
            self.0 = Some(DateFilterType::default_between());
        }
    }
}
