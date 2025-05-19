use std::ops::Deref;

use egui::{Grid, Ui, Widget};

use crate::model::transactions::Transaction;

pub struct TransactionFullView {
    record: Transaction,
}

impl TransactionFullView {
    pub fn new(record: Transaction) -> Self {
        Self { record }
    }

    fn description_ui(&mut self, ui: &mut Ui) {
        ui.heading("Description:");
        //ui.vertical(|ui| match self.description_container() {
        //    Some(desc) => {
        //        Grid::new("description_grid").show(ui, |ui| {
        //            desc.as_vec().into_iter().for_each(|desc| {
        //                ui.label(&desc.title);
        //                ui.label(
        //                    desc.datetime_created
        //                        .format("%d/%m/%Y %H:%M")
        //                        .to_string(),
        //                );
        //                ui.label(&desc.desc);
        //                ui.end_row();
        //            });
        //        });
        //    }
        //    _ => {
        //        ui.label("no desc");
        //    }
        //});
    }
}

impl Widget for &mut TransactionFullView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical_centered(|ui| {
            Grid::new(self.uuid).spacing([10., 10.]).show(ui, |ui| {
                ui.heading("Expense Record: ");
                ui.heading(self.uuid.hyphenated().to_string());
                ui.end_row();

                ui.heading("Time of transaction:");
                ui.heading(
                    self.datetime().format("%d/%m/%Y %H:%M").to_string(),
                );
                ui.end_row();

                ui.heading("Amount:");
                ui.heading(self.amount().to_string());
                ui.end_row();

                //ui.heading("Tags:");
                //ui.horizontal_wrapped(|ui| {
                //    self.tags().iter().for_each(|tag| {
                //        ui.group(|ui| {
                //            ui.label(tag);
                //        });
                //    });
                //});
                //ui.end_row();

                self.description_ui(ui);
            });
        })
        .response
    }
}

impl Deref for TransactionFullView {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        &self.record
    }
}
