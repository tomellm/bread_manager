use egui::Ui;

use crate::model::records::ExpenseRecord;

use super::{box_dyn, DataFilter, TableFilter};

#[derive(Default)]
pub struct AmountFilter(Option<AmountFilterType>);

#[derive(Clone)]
enum AmountFilterType {
    Precise(f64),
    Between(f64, f64),
}
impl AmountFilterType {
    fn default_precise() -> Self {
        Self::Precise(0.)
    }
    fn default_between() -> Self {
        Self::Between(0., 0.)
    }
}

impl TableFilter for AmountFilter {
    fn name(&self) -> &str {
        "Amount"
    }
    fn active(&self) -> bool {
        self.0.is_some()
    }
    fn deactivate(&mut self) {
        self.0 = None;
    }
    fn display(&mut self, ui: &mut Ui) {
        if let Some(amount_filter) = &mut self.0 {
            match amount_filter {
                AmountFilterType::Precise(amount) => {
                    ui.add(egui::DragValue::new(amount).max_decimals(2));
                }
                AmountFilterType::Between(lower, upper) => {
                    ui.label("lower");
                    ui.add(egui::DragValue::new(lower).max_decimals(2));
                    ui.label("upper");
                    ui.add(egui::DragValue::new(upper).max_decimals(2));
                }
            }
        }
    }
    fn filter(&self) -> Option<DataFilter> {
        self.0.as_ref().map(|amount_filter| match amount_filter {
            AmountFilterType::Precise(amount) => {
                let amount = to_isize(*amount);
                box_dyn(move |record: &ExpenseRecord| {
                    record.amount().eq(&amount)
                })
            }
            AmountFilterType::Between(lower, upper) => {
                let lower = to_isize(*lower);
                let upper = to_isize(*upper);
                box_dyn(move |record: &ExpenseRecord| {
                    lower <= *record.amount() && record.amount() <= &upper
                })
            }
        })
    }
    fn filter_activation(&mut self, ui: &mut Ui) {
        if ui.button("precise").clicked() {
            self.0 = Some(AmountFilterType::default_precise());
        }
        if ui.button("between").clicked() {
            self.0 = Some(AmountFilterType::default_between());
        }
    }
}

fn to_isize(amount: f64) -> isize {
    (amount * 100.) as isize
}
