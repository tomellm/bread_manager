use std::cmp::Ordering;

use egui::Ui;
use egui_plot::{Line, Plot, PlotPoints};
use uuid::Uuid;

use crate::{model::records::ExpenseRecord, utils::communicator::Communicator};


pub struct Visualizations {
    update_callback_ctx: Option<egui::Context>,
    records_communicator: Communicator<Uuid, ExpenseRecord>,
}

impl eframe::App for Visualizations {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_callback_ctx = Some(ctx.clone());
        self.records_communicator.update();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Visualizations");
            self.expenses_line(ui);
        });
        
    }
}

impl Visualizations {
    pub fn new(
        records_communicator: Communicator<Uuid, ExpenseRecord>,
    ) -> Self {
        Self { 
            update_callback_ctx: None,
            records_communicator
        }
    }


    fn expenses_line(&self, ui: &mut Ui) {
        /*
            let expenses_map = self.records_communicator.view();
            let possible_min_date = expenses_map.into_iter()
                .reduce(|(aid, a), (bid, b)| {
                    match a.datetime().cmp(b.datetime()) {
                        Ordering::Less => (bid, b),
                        _ => (aid, a)
                    }
                });
            let Some((_, min_record)) = possible_min_date else {
                return;
            };


            let expenses: PlotPoints = expenses_map.iter()
                .map(|(_, record)| {
                    let time_diff = (
                        *record.datetime() - *min_record.datetime()
                    ).num_days();
                    [time_diff, record.]
                }).collect();

            let sin: PlotPoints = (0..1000).map(|i| {
                let x = i as f64 * 0.01;
                [x, x.sin()]
            }).collect();
            let line = Line::new(sin);
            Plot::new("my_plot").view_aspect(2.0).show(ui, |plot_ui| plot_ui.line(line));
            */
    }
}
