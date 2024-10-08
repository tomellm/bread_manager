mod bar_chart;

use bar_chart::BarChartVis;
use data_communicator::buffered::{communicator::Communicator, query::QueryType};
use eframe::App;
use uuid::Uuid;

use crate::model::records::ExpenseRecord;

pub struct Visualizations {
    update_callback_ctx: Option<egui::Context>,
    bars: BarChartVis,
    selected_anchor: Anchor,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum Anchor {
    BarChart,
}

impl App for Visualizations {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_callback_ctx = Some(ctx.clone());

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Visualizations");
            match self.selected_anchor {
                Anchor::BarChart => self.bars.view(ui),
            }
        });
    }
}

impl Visualizations {
    pub fn init(
        records: Communicator<Uuid, ExpenseRecord>,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            let _ = records.query_future(QueryType::All).await;
            let bars = BarChartVis::new(records);

            Self {
                update_callback_ctx: None,
                bars,
                selected_anchor: Anchor::BarChart,
            }
        }
    }
}
