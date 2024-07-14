use egui::Ui;
use uuid::Uuid;

use crate::{model::records::ExpenseRecord, utils::communicator::Communicator};

pub struct TableView {
    records_communicator: Communicator<Uuid, ExpenseRecord>,
    column_toggles: ColumnToggles,
}

impl eframe::App for TableView {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("table view");
                if ui.button("delete all").clicked() {
                    self.delete_all();
                }
            });

            self.column_toggles(ui);

            egui::ScrollArea::both().show(ui, |ui| {
                egui::Grid::new("table of records").show(ui, |ui| {
                    if self.show_datetime_created() { ui.label("datetime created"); }
                    if self.show_datetime() { ui.label("datetime"); }
                    if self.show_uuid() { ui.label("uuid"); }
                    if self.show_amount() { ui.label("amount"); }
                    if self.show_description() { ui.label("description"); }
                    if self.show_tags() { ui.label("tags"); }
                    if self.show_origin() { ui.label("origin"); }
                    ui.end_row();
                    for (_, record) in self.records_communicator.view().iter() {
                        if self.show_datetime_created() { ui.label(format!("{}", record.created().date_naive())); } 
                        if self.show_datetime() { ui.label(format!("{}", record.datetime().date_naive())); }
                        if self.show_uuid() { ui.label(format!("{}", record.uuid().0)); } 
                        if self.show_amount() { ui.label(record.formatted_amount()); }
                        if self.show_description() { ui.label(format!("{:?}", record.description())); }
                        if self.show_tags() { ui.label(format!("{:?}", record.tags())); }
                        if self.show_origin() { ui.label(record.origin().to_string()); }
                        ui.end_row();
                    }
                });
            });
        });
    }
}

impl TableView {
    pub fn new(records_communicator: Communicator<Uuid, ExpenseRecord>) -> Self {

        Self {
            records_communicator,
            column_toggles: ColumnToggles::default()
        }
    }
    pub fn show_file_viewer() -> bool {
        false
    }

    pub fn delete_all(&mut self) {
        let keys = self
            .records_communicator
            .view()
            .iter()
            .map(|(uuid, _)| *uuid)
            .collect::<Vec<_>>();
        self.records_communicator.delete_many(keys);
    }

    fn column_toggles(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            for (label, boolean) in self.column_toggles.toggles() {
                ui.horizontal_wrapped(|ui| {
                    ui.checkbox(boolean, label)
                });
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
    datetime:  bool,
    origin: bool,
}

impl Default for ColumnToggles {
    fn default() -> Self {
        Self { 
            datetime_created: false, 
            uuid: false, 
            amount: true, 
            description: true, 
            tags: false, 
            datetime: true, 
            origin: false
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
