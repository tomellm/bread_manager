mod bar_chart;

use bar_chart::BarChartVis;
use diesel::{QueryDsl, SelectableHelper};
use eframe::App;
use hermes::factory::{self, Factory};
use uuid::Uuid;

use crate::{
    db::records::{DbRecord, RECORDS_FROM_DB_FN},
    model::records::ExpenseRecord,
    schema::expense_records::dsl::expense_records as records_table,
};

use super::DbConn;

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
        factory: Factory<DbConn>,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            let mut records = factory.builder().projector_arc(RECORDS_FROM_DB_FN.clone());
            let _ = records.query(|| records_table.select(DbRecord::as_select()));

            let bars = BarChartVis::new(records);

            Self {
                update_callback_ctx: None,
                bars,
                selected_anchor: Anchor::BarChart,
            }
        }
    }
}
