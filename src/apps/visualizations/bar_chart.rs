use std::{cmp::Ordering, collections::HashMap, ops::Sub};

use chrono::{DateTime, Days, Local};
use egui::Ui;
use egui_plot::{Bar, BarChart, Plot};
use tokio::sync::watch;
use uuid::Uuid;

use crate::model::records::ExpenseRecord;

pub(super) struct BarChartVis {
    values: watch::Receiver<HashMap<Uuid, ExpenseRecord>>,
    weekly: Vec<Bar>,
    monthly: Vec<Bar>,
}

impl BarChartVis {
    pub fn new(values: watch::Receiver<HashMap<Uuid, ExpenseRecord>>) -> Self {
        let (weekly, monthly) = Self::update_graphs(&values.borrow());
        Self { values, weekly, monthly }
    }

    pub fn update(&mut self) {
        if self
            .values
            .has_changed()
            .expect("This Reciver should never return an error.")
        {
            let (weekly, monthly) = Self::update_graphs(&self.values.borrow());
            self.weekly = weekly;
            self.monthly = monthly;
        }
    }

    fn update_graphs(records: &HashMap<Uuid, ExpenseRecord>) -> (Vec<Bar>, Vec<Bar>) {
        println!("{}", records.len());
        if records.is_empty() {
            return (vec![], vec![]);
        }

        let min = records
            .iter()
            .min_by(|a, b| a.1.datetime().cmp(b.1.datetime()))
            .expect("No min Record found but should be present")
            .1
            .datetime();

        let max = records
            .iter()
            .max_by(|a, b| a.1.datetime().cmp(b.1.datetime()))
            .expect("No max Record found but should be present")
            .1
            .datetime();

        let mut weekly_amounts = (0..max.sub(min).num_weeks())
            .into_iter()
            .map(|week_index| (week_index as u64 * 7, 0i64))
            .collect::<Vec<_>>();

        let is_in_week = |weeks_from_in_days: u64, to_cmp: &DateTime<Local>| -> bool {
            let lower = min
                .checked_add_days(Days::new(weeks_from_in_days))
                .expect("Adding days shouldnt fail");
            let upper = min
                .checked_add_days(Days::new(weeks_from_in_days + 7))
                .expect("Adding days shouldnt fail");
            match (lower.cmp(to_cmp), upper.cmp(to_cmp)) {
                (Ordering::Equal, Ordering::Greater) | (Ordering::Less, Ordering::Greater) => true,
                _ => false,
            }
        };

        records.into_iter().for_each(|(_, record)| {
            weekly_amounts.iter_mut().for_each(|(week_index_in_days, amount)| {
                if is_in_week(*week_index_in_days, record.datetime()) {
                    *amount += *record.amount() as i64;
                }
            })
        });

        let bars = weekly_amounts.into_iter().map(|(week_index_as_days, amount)| 
            Bar::new(week_index_as_days as f64 / 7f64, amount as f64 / 100.0)
                .name(format!("{}", min.checked_add_days(Days::new(week_index_as_days)).expect("Adding days shouldnt fail")))
        ).collect::<Vec<_>>();

        (bars, vec![])
    }

    pub fn view(&mut self, ui: &mut Ui) {
        self.update();

        let weekly = self.weekly.clone();

        ui.label("weekly bar chart");
        Plot::new("my_plot").view_aspect(2.0).show(ui, |plot_ui| {
            plot_ui.bar_chart(BarChart::new(weekly));
            //plot_ui.line(Line::new(ordered_list.into_iter().collect::<PlotPoints>()))
        });
    }
}
