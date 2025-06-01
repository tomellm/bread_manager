pub mod actions;
mod filterstate;

use actions::ActionState;
use eframe::App;
use egui::{CentralPanel, SidePanel};
use egui_light_states::UiStates;
use filterstate::FilterState;
use hermes::{
    carrier::execute::ImplExecuteCarrier,
    container::{data::ImplData, manual},
    factory::Factory,
};

use crate::{
    components::expense_records::table::RecordsTable,
    model::transactions::Transaction,
};

pub struct TableView {
    transacts: manual::Container<Transaction>,
    columns_info: RecordsTable,

    filter_state: FilterState,
    hide_filters: bool,
    action_state: ActionState,

    side_panel_state: SidePanelState,
    states: UiStates,
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
enum SidePanelState {
    #[default]
    Filters,
    Actions,
}

impl SidePanelState {
    fn values() -> [SidePanelState; 2] {
        [SidePanelState::Filters, SidePanelState::Actions]
    }
}

impl App for TableView {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.transacts.state_update(true);

        CentralPanel::default().show(ctx, |ui| {
            CentralPanel::default().show_inside(ui, |ui| {
                if self.transacts.data().is_empty() {
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
                        //self.records.execute(DbRecord::delete_many());
                        //self.records.execute(DbDataImport::delete_many());
                    }

                    ui.label(format!(
                        "Curretly {} records.",
                        self.transacts.data().len()
                    ));

                    if self.hide_filters && ui.button("filters").clicked() {
                        self.hide_filters = false;
                    }
                });

                self.columns_info.toggles(ui);

                self.columns_info.show_filtered(
                    &mut self.transacts,
                    |r| self.filter_state.filter(r),
                    ui,
                );
            });
            if !self.transacts.data().is_empty() && !self.hide_filters {
                SidePanel::right("filter_selection")
                    .resizable(true)
                    .show_inside(ui, |ui| {
                        ui.horizontal(|ui| {
                            SidePanelState::values().into_iter().for_each(
                                |val| {
                                    ui.add_enabled_ui(
                                        val != self.side_panel_state,
                                        |ui| {
                                            if ui
                                                .button(format!("{val:?}"))
                                                .clicked()
                                            {
                                                self.side_panel_state = val;
                                            }
                                        },
                                    );
                                },
                            );
                            if ui.button(">>").clicked() {
                                self.hide_filters = true;
                            }
                        });
                        ui.separator();

                        match self.side_panel_state {
                            SidePanelState::Filters => {
                                self.filter_state.display_filters(ui)
                            }
                            SidePanelState::Actions => {
                                self.action_state.display_actions(
                                    &mut self.transacts,
                                    |r| self.filter_state.filter(r),
                                    ui,
                                )
                            }
                        };
                    });
            }
        });
    }
}

impl TableView {
    pub fn init(
        factory: Factory,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            let records = factory.builder().file(file!()).manual();
            //records.stored_query(DbRecord::find_all_active());
            Self {
                action_state: ActionState::new(records.actor()),
                transacts: records,
                columns_info: RecordsTable::default(),
                filter_state: FilterState::default(),
                hide_filters: true,
                side_panel_state: SidePanelState::default(),
                states: UiStates::default(),
            }
        }
    }
    pub fn show_file_viewer() -> bool {
        false
    }

    //pub fn delete_all(&mut self) -> ImmediateValuePromise<ChangeResult> {
    //    let keys = self.records.data.keys_cloned();
    //    ImmediateValuePromise::new(self.records.delete_many(keys))
    //}
}

const NO_RECORDS_EMPTY_TEXT: &str = r#"
Usually there would be a list of expenses here...

You have not yet added any expenses to the Application. First create a profile in the Profiles tab,
then parse a file with it in the File Upload tab and finally view the expenses here.
"#;
