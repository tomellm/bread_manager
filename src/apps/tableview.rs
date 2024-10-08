use data_communicator::buffered::{communicator::Communicator, query::QueryType};
use eframe::App;
use egui::{Frame, Label, RichText, Sense, Ui, UiBuilder, Widget};
use tracing::info;
use uuid::Uuid;

use crate::{components::soft_button::soft_button, model::records::ExpenseRecord};

pub struct TableView {
    records: Communicator<Uuid, ExpenseRecord>,
    column_toggles: ColumnToggles,
}

impl App for TableView {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.records.state_update();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("table view");
                if ui.button("delete all").clicked() {
                    self.delete_all();
                }
                ui.label(format!("Curretly {} records.", self.records.data().len()));
            });

            self.column_toggles(ui);

            egui::ScrollArea::both().show(ui, |ui| {
                egui::Grid::new("table of records").show(ui, |ui| {
                    if self.show_datetime_created() {
                        ui.label("datetime created");
                    }
                    if self.show_datetime() {
                        let response = soft_button("datetime_sorting", "datetime", ui);
                        if response.double_clicked() {
                            self.records.sort(|a, b| b.datetime().cmp(a.datetime()));
                        } else if response.clicked() {
                            self.records.sort(|a, b| a.datetime().cmp(b.datetime()));
                        } 
                    }
                    if self.show_uuid() {
                        let response = soft_button("uuid_sorting", "uuid", ui);
                        if response.double_clicked() {
                            self.records.sort(|a, b| b.uuid().cmp(a.uuid()));
                        } else if response.clicked() {
                            self.records.sort(|a, b| a.uuid().cmp(b.uuid()));
                        } 
                    }
                    if self.show_amount() {
                        let response = soft_button("amount_sorting", "amount", ui);
                        if response.double_clicked() {
                            self.records.sort(|a, b| b.amount().cmp(a.amount()));
                        } else if response.clicked() {
                            self.records.sort(|a, b| a.amount().cmp(b.amount()));
                        } 
                    }
                    if self.show_description() {
                        ui.label("description");
                    }
                    if self.show_tags() {
                        ui.label("tags");
                    }
                    if self.show_origin() {
                        let response = soft_button("origin_sorting", "origin", ui);
                        if response.double_clicked() {
                            self.records.sort(|a, b| b.origin().cmp(a.origin()));
                        } else if response.clicked() {
                            self.records.sort(|a, b| a.origin().cmp(b.origin()));
                        } 
                    }
                    ui.end_row();
                    for record in self.records.data_sorted() {
                        if self.show_datetime_created() {
                            ui.label(format!("{}", record.created().date_naive()));
                        }
                        if self.show_datetime() {
                            ui.label(format!("{}", record.datetime().date_naive()));
                        }
                        if self.show_uuid() {
                            ui.label(format!("{}", record.uuid().0));
                        }
                        if self.show_amount() {
                            ui.label(record.formatted_amount());
                        }
                        if self.show_description() {
                            ui.label(format!("{:?}", record.description()));
                        }
                        if self.show_tags() {
                            ui.label(format!("{:?}", record.tags()));
                        }
                        if self.show_origin() {
                            ui.label(record.origin().to_string());
                        }
                        ui.end_row();
                    }
                });
            });
        });
    }
}

impl TableView {
    pub fn init(
        records: Communicator<Uuid, ExpenseRecord>,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            let _ = records.query_future(QueryType::All).await;
            Self {
                records,
                column_toggles: ColumnToggles::default(),
            }
        }
    }
    pub fn show_file_viewer() -> bool {
        false
    }

    pub fn delete_all(&mut self) {
        let keys = self.records.data_map().keys().cloned().collect::<Vec<_>>();
        self.records.delete_many(keys);
    }

    fn column_toggles(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            for (label, boolean) in self.column_toggles.toggles() {
                ui.horizontal_wrapped(|ui| ui.checkbox(boolean, label));
            }
        });
    }

    fn show_datetime_created(&self) -> bool {
        self.column_toggles.datetime_created
    }

    fn show_uuid(&self) -> bool {
        self.column_toggles.uuid
    }

    fn show_amount(&self) -> bool {
        self.column_toggles.amount
    }

    fn show_description(&self) -> bool {
        self.column_toggles.description
    }

    fn show_tags(&self) -> bool {
        self.column_toggles.tags
    }

    fn show_datetime(&self) -> bool {
        self.column_toggles.datetime
    }

    fn show_origin(&self) -> bool {
        self.column_toggles.origin
    }
}

struct ColumnToggles {
    datetime_created: bool,
    uuid: bool,
    amount: bool,
    description: bool,
    tags: bool,
    datetime: bool,
    origin: bool,
}

impl Default for ColumnToggles {
    fn default() -> Self {
        Self {
            datetime_created: false,
            uuid: false,
            amount: true,
            description: true,
            tags: true,
            datetime: true,
            origin: true,
        }
    }
}

impl ColumnToggles {
    fn toggles(&mut self) -> [(&str, &mut bool); 7] {
        [
            ("datetime created", &mut self.datetime_created),
            ("uuid", &mut self.uuid),
            ("amount", &mut self.amount),
            ("description", &mut self.description),
            ("tags", &mut self.tags),
            ("datetime", &mut self.datetime),
            ("origin", &mut self.origin),
        ]
    }
}
