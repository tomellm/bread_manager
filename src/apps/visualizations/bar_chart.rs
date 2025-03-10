use std::{cmp::Ordering, ops::Sub};

use chrono::{DateTime, Datelike, Days, Local, Months};
use egui::Ui;
use egui_plot::{Bar, BarChart, Plot};
use hermes::{
    carrier::query::ImplQueryCarrier,
    container::{data::ImplData, projecting::ProjectingContainer},
    factory::Factory,
};

use crate::{db::records::DbRecord, model::records::ExpenseRecord};

#[derive(Default)]
enum Charts {
    #[default]
    Weekly,
    Monthly,
}

pub(super) struct BarChartVis {
    selected: Charts,
    records: ProjectingContainer<ExpenseRecord, DbRecord>,
    weekly: Vec<Bar>,
    monthly: Vec<Bar>,
}

impl BarChartVis {
    pub fn new(factory: &Factory) -> impl std::future::Future<Output = Self> + Send + 'static {
        let mut records = factory.builder().name("bar_chart_vis_records").projector();
        async move {
            records.stored_query(DbRecord::find_all_active());
            let (weekly, monthly) = Self::update_graphs(&vec![]);
            Self {
                selected: Charts::default(),
                records,
                weekly,
                monthly,
            }
        }
    }

    pub fn update(&mut self) {
        self.records.state_update(true);
        if self.records.has_changed() {
            let (weekly, monthly) = Self::update_graphs(self.records.set_viewed().data());
            self.weekly = weekly;
            self.monthly = monthly;
        }
    }

    fn update_graphs(records: &Vec<ExpenseRecord>) -> (Vec<Bar>, Vec<Bar>) {
        if records.is_empty() {
            return (vec![], vec![]);
        }

        let min = records
            .iter()
            .min_by(|a, b| a.datetime().cmp(b.datetime()))
            .expect("No min Record found but should be present")
            .datetime();

        let max = records
            .iter()
            .max_by(|a, b| a.datetime().cmp(b.datetime()))
            .expect("No max Record found but should be present")
            .datetime();

        let mut weekly_amounts = (0..max.sub(min).num_weeks())
            .map(|week_index| (week_index as u64 * 7, 0f64))
            .collect::<Vec<_>>();

        let mut monthly_amounts = {
            // Takes the inbetween years to calc the whole years inbetween and then
            // adds the two ends from the starting date to the ending date.
            let months_diff =
                ((max.year() - min.year()) as u32 * 12) + (12 - min.month()) + max.month();
            let first_month = min.clone().with_day(1).unwrap();

            (0..months_diff)
                .map(|month_index| {
                    let bar_month = first_month
                        .checked_add_months(Months::new(month_index))
                        .unwrap();
                    (bar_month, 0f64)
                })
                .collect::<Vec<_>>()
        };

        let is_in_week = is_in_week_fn(min);

        for record in records {
            for (week_index_in_days, amount) in &mut weekly_amounts {
                if is_in_week(*week_index_in_days, record.datetime()) {
                    *amount += record.amount_euro_f64();
                }
            }
            for (month, amount) in &mut monthly_amounts {
                if is_in_month(month, record.datetime()) {
                    *amount += record.amount_euro_f64();
                }
            }
        }

        let week_bars = weekly_amounts
            .into_iter()
            .map(|(week_index_as_days, amount)| {
                let date = min
                    .checked_add_days(Days::new(week_index_as_days))
                    .expect("Adding days shouldnt fail")
                    .format("%e %B %Y");
                let bar_name = format!("{date} {amount:.2}€");
                Bar::new(week_index_as_days as f64 / 7f64, amount).name(bar_name)
            })
            .collect::<Vec<_>>();
        let month_bars = monthly_amounts
            .into_iter()
            .enumerate()
            .map(|(index, (month, amount))| {
                let date = month.format("%B %Y");
                let bar_name = format!("{date} {amount:.2}€");
                Bar::new(index as f64, amount).name(bar_name)
            })
            .collect::<Vec<_>>();

        (week_bars, month_bars)
    }

    pub fn view(&mut self, ui: &mut Ui) {
        self.update();

        ui.horizontal(|ui| {
            if ui.button("weekly").clicked() {
                self.selected = Charts::Weekly;
            }
            if ui.button("monthly").clicked() {
                self.selected = Charts::Monthly;
            }
        });

        ui.label(format!("Curretly {} records.", self.records.data().len()));
        match self.selected {
            Charts::Weekly => {
                Plot::new("weekly_plot")
                    .view_aspect(2.0)
                    .show(ui, |plot_ui| {
                        plot_ui.bar_chart(BarChart::new(self.weekly.clone()))
                    });
            }
            Charts::Monthly => {
                Plot::new("monthly_plot")
                    .view_aspect(2.0)
                    .show(ui, |plot_ui| {
                        plot_ui.bar_chart(BarChart::new(self.monthly.clone()))
                    });
            }
        }
    }
}

fn is_in_week_fn(min: &DateTime<Local>) -> impl Fn(u64, &DateTime<Local>) -> bool + '_ {
    |weeks_from_in_days: u64, to_cmp: &DateTime<Local>| -> bool {
        let lower = min
            .checked_add_days(Days::new(weeks_from_in_days))
            .expect("Adding days shouldnt fail");
        let upper = min
            .checked_add_days(Days::new(weeks_from_in_days + 7))
            .expect("Adding days shouldnt fail");
        matches!(
            (lower.cmp(to_cmp), upper.cmp(to_cmp)),
            (Ordering::Equal | Ordering::Less, Ordering::Greater)
        )
    }
}

fn is_in_month(month: &DateTime<Local>, to_cmp: &DateTime<Local>) -> bool {
    matches!(
        (
            to_cmp.year().cmp(&month.year()),
            to_cmp.month().cmp(&month.month())
        ),
        (Ordering::Equal, Ordering::Equal)
    )
}
