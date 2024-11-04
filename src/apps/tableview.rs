mod filterstate;

use diesel::{QueryDsl, SelectableHelper};
use eframe::App;
use egui::{CentralPanel, SidePanel};
use egui_light_states::{future_await::FutureAwait, UiStates};
use filterstate::FilterState;
use hermes::{container::projecting::ProjectingContainer, factory::{self, Factory}};
use lazy_async_promise::ImmediateValuePromise;
use uuid::Uuid;

use crate::{
    components::expense_records::table::RecordsTable,
    db::records::{DbRecord, RECORDS_FROM_DB_FN},
    model::records::ExpenseRecord,
    schema::expense_records::dsl::expense_records as records_table,
};

use super::DbConn;

pub struct TableView {
    records: ProjectingContainer<ExpenseRecord, DbRecord, DbConn>,
    columns_info: RecordsTable,
    filter_state: FilterState,
    states: UiStates,
}

impl App for TableView {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.records.state_update();

        CentralPanel::default().show(ctx, |ui| {
            CentralPanel::default().show_inside(ui, |ui| {
                if self.records.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(40.);
                        ui.label(NO_RECORDS_EMPTY_TEXT);
                        ui.add_space(40.);
                    });
                    return;
                }

                ui.horizontal(|ui| {
                    ui.label("table view");
                    if ui.button("delete all").clicked() {
                        let promise = self.delete_all();
                        self.states.set_future("delete_all_records").set(promise);
                    }
                    self.states
                        .future_status::<ChangeResult>("delete_all_records")
                        .only_poll();

                    ui.label(format!("Curretly {} records.", self.records.data().len()));
                });

                self.columns_info.toggles(ui);

                self.columns_info.show_filtered(
                    &mut self.records,
                    |r| self.filter_state.filter(r),
                    ui,
                );
            });
            if !self.records.is_empty() {
                SidePanel::right("filter_selection")
                    .resizable(true)
                    .show_inside(ui, |ui| {
                        self.filter_state.display_filters(ui);
                    });
            }
        });
    }
}

impl TableView {
    pub fn init(
        factory: Factory<DbConn>,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            let mut records = factory.builder().projector_arc(RECORDS_FROM_DB_FN.clone());
            let _ = records.query(|| records_table.select(DbRecord::as_select()));
            Self {
                records,
                columns_info: RecordsTable::default(),
                filter_state: FilterState::default(),
                states: UiStates::default(),
            }
        }
    }
    pub fn show_file_viewer() -> bool {
        false
    }

    pub fn delete_all(&mut self) -> ImmediateValuePromise<ChangeResult> {
        let keys = self.records.data.keys_cloned();
        ImmediateValuePromise::new(self.records.delete_many(keys))
    }
}

const NO_RECORDS_EMPTY_TEXT: &str = r#"
Usually there would be a list of expenses here...

You have not yet added any expenses to the Application. First create a profile in the Profiles tab,
then parse a file with it in the File Upload tab and finally view the expenses here.
"#;
