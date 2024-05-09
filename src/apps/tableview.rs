use uuid::Uuid;

use crate::{model::records::ExpenseRecord, utils::communicator::Communicator};


pub struct TableView {
    records_communicator: Communicator<Uuid, ExpenseRecord>
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
            egui::Grid::new("table of records").show(ui, |ui| {
                ui.label("amount");
                ui.label("time");
                ui.label("tags");
                ui.end_row();
                for (_, record) in self.records_communicator.view().iter() {
                    ui.label(format!("{}", record.amount()));
                    ui.label(format!("{}", record.datetime()));
                    ui.label(format!("{:?}", record.tags()));
                    ui.end_row();
                }
            });
        });
    }
}

impl TableView {
    pub fn new(
        records_communicator: Communicator<Uuid, ExpenseRecord>
    ) -> Self {
        Self { records_communicator }
    }
    pub fn show_file_viewer() -> bool {
        return false;
    }

    pub fn delete_all(&mut self) {
        let keys = self.records_communicator.view()
            .iter()
            .map(|(uuid, _)| *uuid)
            .collect::<Vec<_>>();
        self.records_communicator.delete_many(keys);
    }
}
