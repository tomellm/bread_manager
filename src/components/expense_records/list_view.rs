use egui::{Grid, Widget};
use tracing::info;

use crate::model::records::ExpenseRecord;

pub struct RecordListView<'a> {
    record: &'a ExpenseRecord,
}
impl<'a> RecordListView<'a> {
    pub fn new(record: &'a ExpenseRecord) -> Self {
        Self { record }
    }
}

impl<'a> Widget for RecordListView<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let r = self.record;

        Grid::new(format!("expense_record_{}", **r.uuid()))
            .show(ui, |ui| {
                ui.label("Uuid");
                ui.label(r.uuid().to_string());
                ui.end_row();

                ui.label("Amount");
                ui.label(r.amount_euro().to_string());
                ui.end_row();

                ui.label("DateTime performed");
                ui.label(r.datetime().to_string());
                ui.end_row();

                ui.label("Description");
                ui.label("");
                ui.end_row();

                if let Some(descr_cont) = r.description_container() {
                    for desc in descr_cont.as_vec() {
                        ui.label(&desc.title);
                        ui.label(&desc.desc);
                        ui.end_row();
                    }
                }


                ui.label("Tags");
                ui.horizontal_wrapped(|ui| {
                    for tag in r.tags() {
                        ui.group(|ui| {
                            ui.label(tag);
                        });
                    }
                });
                ui.end_row();

                ui.label("Origin");
                ui.label(r.origin());
                ui.end_row();

                ui.label("DateTime created");
                ui.label(r.created().to_string());
                ui.end_row();

                if ui.button("show desc").clicked() {
                    info!("{:?}", r.description_container())
                }
                if ui.button("show cols").clicked() {
                    info!("{:?}", r.data())
                }
                ui.end_row();
            })
            .response
    }
}
