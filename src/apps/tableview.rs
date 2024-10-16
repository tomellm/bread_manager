mod filterstate;
mod tablecolumns;

use data_communicator::buffered::{change::ChangeResult, communicator::Communicator, query::QueryType};
use eframe::App;
use egui::{CentralPanel, SidePanel};
use egui_light_states::{future_await::FutureAwait, UiStates};
use filterstate::FilterState;
use lazy_async_promise::ImmediateValuePromise;
use tablecolumns::TableColumns;
use uuid::Uuid;

use crate::model::records::ExpenseRecord;

pub struct TableView {
    records: Communicator<Uuid, ExpenseRecord>,
    columns_info: TableColumns,
    filter_state: FilterState,
    states: UiStates,
}

impl App for TableView {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.records.state_update();

        CentralPanel::default().show(ctx, |ui| {
            CentralPanel::default().show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("table view");
                    if ui.button("delete all").clicked() {
                        let promise = self.delete_all();
                        self.states.set_future("delete_all_records")
                            .set(promise);
                    }
                    self.states.future_status::<ChangeResult>("delete_all_records")
                        .only_poll();
                        
                    ui.label(format!("Curretly {} records.", self.records.data().len()));
                });

                self.columns_info.toggles(ui);

                egui::ScrollArea::both().show(ui, |ui| {
                    egui::Grid::new("table of records").show(ui, |ui| {
                        self.columns_info.header(&mut self.records, ui);
                        ui.end_row();

                        for record in self
                            .records
                            .data
                            .sorted_iter()
                            .filter(|r| self.filter_state.filter(r))
                        {
                            self.columns_info.row(record, ui);
                            ui.end_row();
                        }
                    });
                });
            });
            SidePanel::right("filter_selection")
                .resizable(true)
                .show_inside(ui, |ui| {
                    self.filter_state.display_filters(ui);
                });
        });
    }
}

impl TableView {
    pub fn init(
        records: Communicator<Uuid, ExpenseRecord>,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            let _ = records.query(QueryType::All).await;
            Self {
                records,
                columns_info: TableColumns::default(),
                filter_state: FilterState::default(),
                states: UiStates::default(),
            }
        }
    }
    pub fn show_file_viewer() -> bool {
        false
    }

    pub fn delete_all(&mut self) -> ImmediateValuePromise<ChangeResult>{
        let keys = self.records.data.keys_cloned();
        ImmediateValuePromise::new(self.records.delete_many(keys))
    }
}
