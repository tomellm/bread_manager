mod bar_chart;

use bar_chart::BarChartVis;
use uuid::Uuid;

use crate::{model::records::ExpenseRecord, utils::communicator::Communicator};

pub struct Visualizations {
    update_callback_ctx: Option<egui::Context>,
    records_communicator: Communicator<Uuid, ExpenseRecord>,
    bars: BarChartVis,
    selected_anchor: Anchor
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum Anchor {
    BarChart
}

impl eframe::App for Visualizations {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_callback_ctx = Some(ctx.clone());
        self.records_communicator.update();
        self.update_visualizations();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Visualizations");
            match self.selected_anchor {
                Anchor::BarChart => self.bars.view(ui),
            }
        });
    }
}

impl Visualizations {
    pub fn new(records_communicator: Communicator<Uuid, ExpenseRecord>) -> Self {
        let bars = BarChartVis::new(records_communicator.viewer());

        Self {
            update_callback_ctx: None,
            records_communicator,
            bars,
            selected_anchor: Anchor::BarChart
        }
    }

    fn update_visualizations(&mut self) {
        
    }
}
