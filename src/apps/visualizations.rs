mod bar_chart;

use bar_chart::BarChartVis;
use eframe::App;
use hermes::factory::Factory;

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
    pub fn init(factory: &Factory) -> impl std::future::Future<Output = Self> + Send + 'static {
        let bars = BarChartVis::new(factory);
        async move {
            Self {
                update_callback_ctx: None,
                bars: bars.await,
                selected_anchor: Anchor::BarChart,
            }
        }
    }
}
