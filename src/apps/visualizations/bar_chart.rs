use std::{
    cmp::Ordering,
    ops::{Deref, Sub},
};

use chrono::{DateTime, Datelike, Days, Local, Months};
use egui::{CentralPanel, SidePanel, Ui};
use egui_plot::{Bar, BarChart, Plot};
use hermes::{
    container::{data::ImplData, manual},
    factory::Factory,
};
use itertools::Itertools;
use tracing::warn;

use crate::{
    apps::visualizations::code::{CodeBlock, err::CodeExecError, filter::CodeFilter},
    db::query::transaction_query::TransactionQuery,
    model::transactions::Transaction,
};

#[derive(Default)]
enum Charts {
    #[default]
    Weekly,
    Monthly,
}

pub(super) struct BarChartVis {
    selected: Charts,
    transactions: manual::Container<Transaction>,
    weekly: Vec<Bar>,
    monthly: Vec<Bar>,
    code: CodeBlock,
}

impl BarChartVis {
    pub fn new(
        factory: &Factory,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        let mut transactions = factory.builder().file(file!()).manual();
        async move {
            transactions.stored_query(TransactionQuery::all);

            Self {
                selected: Charts::default(),
                transactions,
                weekly: vec![],
                monthly: vec![],
                code: Default::default(),
            }
        }
    }

    pub fn try_update_state(&mut self) {
        self.transactions.state_update(true);
        if self.transactions.has_changed() {
            self.transactions.set_viewed();

            self.update_graphs_with_code();
        }
    }

    fn update_graphs_with_code(&mut self) {
        let filter = match self.code.apply_code {
            true => Some(self.code.create_filter()),
            false => None,
        };
        self.update_graphs(filter)
    }

    fn update_graphs(&mut self, opt_filter: Option<CodeFilter>) {
        let transactions = {
            match opt_filter {
                Some(filter) => {
                    let (filtered_trxn, errors): (Vec<_>, Vec<_>) = self
                        .transactions
                        .data()
                        .deref()
                        .iter()
                        .map(|v| {
                            filter
                                .call(v)
                                .map(|ok_val| match ok_val {
                                    true => Some(v),
                                    false => None,
                                })
                                .map_err(|err| CodeExecError::new(v, err))
                        })
                        .partition_result();

                    match errors.is_empty() {
                        true => Ok(filtered_trxn
                            .into_iter()
                            .flatten()
                            .collect_vec()),
                        false => Err(errors),
                    }
                }
                None => {
                    Ok(self.transactions.data().deref().iter().collect_vec())
                }
            }
        };

        if transactions.is_err() || transactions.as_ref().unwrap().is_empty() {
            if let Err(errors) = transactions {
                warn!("a total of {} errors where created while executing the lua code", errors.len());
                self.code.set_errors(errors);
            }
            self.weekly = vec![];
            self.monthly = vec![];
            return;
        }
        let transactions = transactions.unwrap();
        self.code.reset_example_shape(self.transactions.data());

        let min = transactions
            .iter()
            .min_by(|a, b| a.datetime.cmp_datetime(&b.datetime))
            .expect("No min Record found but should be present")
            .datetime();

        let max = transactions
            .iter()
            .max_by(|a, b| a.datetime.cmp_datetime(&b.datetime))
            .expect("No max Record found but should be present")
            .datetime();

        let mut weekly_amounts = (0..max.sub(min).num_weeks())
            .map(|week_index| (week_index as u64 * 7, 0f64))
            .collect::<Vec<_>>();

        let mut monthly_amounts = {
            // Takes the inbetween years to calc the whole years inbetween and then
            // adds the two ends from the starting date to the ending date.
            let months_diff = ((max.year() - min.year()) as u32 * 12)
                + (12 - min.month())
                + max.month();
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

        for transac in transactions {
            for (week_index_in_days, amount) in &mut weekly_amounts {
                if is_in_week(*week_index_in_days, transac.datetime()) {
                    *amount += transac.amount();
                }
            }
            for (month, amount) in &mut monthly_amounts {
                if is_in_month(month, transac.datetime()) {
                    *amount += transac.amount();
                }
            }
        }

        self.weekly = weekly_amounts
            .into_iter()
            .map(|(week_index_as_days, amount)| {
                let date = min
                    .checked_add_days(Days::new(week_index_as_days))
                    .expect("Adding days shouldnt fail")
                    .format("%e %B %Y");
                let bar_name = format!("{date} {amount:.2}€");
                Bar::new(week_index_as_days as f64 / 7f64, amount)
                    .name(bar_name)
            })
            .collect::<Vec<_>>();
        self.monthly = monthly_amounts
            .into_iter()
            .enumerate()
            .map(|(index, (month, amount))| {
                let date = month.format("%B %Y");
                let bar_name = format!("{date} {amount:.2}€");
                Bar::new(index as f64, amount).name(bar_name)
            })
            .collect::<Vec<_>>();
    }

    pub fn update(&mut self, ui: &mut Ui) {
        self.try_update_state();

        ui.horizontal(|ui| {
            if ui.button("weekly").clicked() {
                self.selected = Charts::Weekly;
            }
            if ui.button("monthly").clicked() {
                self.selected = Charts::Monthly;
            }
            ui.separator();
            ui.label("filter");
            if ui.button("create").clicked() {
                self.code.toggle_show();
            }
            if ui
                .checkbox(&mut self.code.apply_code, "apply code")
                .clicked()
            {
                self.update_graphs_with_code();
            }
            ui.add_enabled_ui(self.code.apply_code, |ui| {
                if ui.button("reapply").clicked() {
                    let filter = self.code.create_filter();
                    self.update_graphs(Some(filter));
                }
            });
        });

        ui.label(format!(
            "Curretly {} records.",
            self.transactions.data().len()
        ));
        CentralPanel::default().show_inside(ui, |ui| match self.selected {
            Charts::Weekly => {
                Plot::new("weekly_plot")
                    .view_aspect(2.0)
                    .show(ui, |plot_ui| {
                        plot_ui.bar_chart(BarChart::new(
                            "weekly_plot",
                            self.weekly.clone(),
                        ))
                    });
            }
            Charts::Monthly => {
                Plot::new("monthly_plot").view_aspect(2.0).show(
                    ui,
                    |plot_ui| {
                        plot_ui.bar_chart(BarChart::new(
                            "monthly_plot",
                            self.monthly.clone(),
                        ))
                    },
                );
            }
        });

        if self.code.show() {
            SidePanel::right("code_filter").resizable(true).show_inside(
                ui,
                |ui| {
                    self.code.update(ui);
                },
            );
        }
    }
}

fn is_in_week_fn(
    min: &DateTime<Local>,
) -> impl Fn(u64, &DateTime<Local>) -> bool + '_ {
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
