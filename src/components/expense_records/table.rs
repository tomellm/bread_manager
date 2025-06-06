use chrono::{DateTime, Local};
use egui::Ui;
use hermes::container::data::ImplData;
use uuid::Uuid;

use crate::{components::table::TableColumn, model::transactions::Transaction};

pub(crate) struct TransactsTable {
    datetime_created: TableColumn<Transaction, DateTime<Local>>,
    uuid: TableColumn<Transaction, Uuid>,
    amount: TableColumn<Transaction, i32>,
    //description: TableColumn<Transaction, String>,
    //tags: TableColumn<Transaction, Vec<String>>,
    datetime: TableColumn<Transaction, DateTime<Local>>,
    //origin: TableColumn<Transaction, String>,
}

impl TransactsTable {
    pub(crate) fn toggles(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.datetime_created.toggle(ui);
            self.datetime.toggle(ui);
            self.uuid.toggle(ui);
            self.amount.toggle(ui);
            //self.description.toggle(ui);
            //self.tags.toggle(ui);
            //self.origin.toggle(ui);
        });
    }
    pub(crate) fn header(&self, ui: &mut Ui) {
        self.datetime_created.header(ui);
        self.datetime.header(ui);
        self.uuid.header(ui);
        self.amount.header(ui);
        //self.description.header(ui);
        //self.tags.header(ui);
        //self.origin.header(ui);
    }
    pub(crate) fn sorting_header(
        &self,
        records: &mut impl ImplData<Transaction>,
        ui: &mut Ui,
    ) {
        self.datetime_created.sorting_header(records, ui);
        self.datetime.sorting_header(records, ui);
        self.uuid.sorting_header(records, ui);
        self.amount.sorting_header(records, ui);
        //self.description.sorting_header(records, ui);
        //self.tags.sorting_header(records, ui);
        //self.origin.sorting_header(records, ui);
    }
    pub(crate) fn row(&self, record: &Transaction, ui: &mut Ui) {
        self.datetime_created.display_value(record, ui);
        self.datetime.display_value(record, ui);
        self.uuid.display_value(record, ui);
        self.amount.display_value(record, ui);
        //self.description.display_value(record, ui);
        //self.tags.display_value(record, ui);
        //self.origin.display_value(record, ui);
    }
    pub(crate) fn show(&self, records: &Vec<Transaction>, ui: &mut Ui) {
        egui::ScrollArea::both().show(ui, |ui| {
            egui::Grid::new("table_of_records").show(ui, |ui| {
                self.header(ui);
                ui.end_row();

                for record in records {
                    self.row(record, ui);
                    ui.end_row();
                }
            });
        });
    }
    pub(crate) fn show_filtered(
        &self,
        records: &mut impl ImplData<Transaction>,
        filter: impl FnMut(&&Transaction) -> bool,
        ui: &mut Ui,
    ) {
        egui::ScrollArea::both().show(ui, |ui| {
            egui::Grid::new("table_of_records")
                .striped(true)
                .show(ui, |ui| {
                    self.sorting_header(records, ui);
                    ui.end_row();

                    records.sorted().into_iter().filter(filter).for_each(
                        |record| {
                            self.row(record, ui);
                            ui.end_row();
                        },
                    );
                });
        });
    }
}

impl Default for TransactsTable {
    fn default() -> Self {
        Self {
            datetime_created: TableColumn::inactive(
                "datetime created",
                d_created,
            ),
            uuid: TableColumn::active("uuid", d_uuid).extract_fn(uuid),
            amount: TableColumn::active("amount", d_amount).extract_fn(amount),
            //description: TableColumn::active("description", d_description),
            //tags: TableColumn::active("tags", d_tags),
            datetime: TableColumn::active("datetime", d_datetime)
                .extract_fn(datetime),
            //origin: TableColumn::active("origin", d_origin).extract_fn(origin),
        }
    }
}

fn d_created(record: &Transaction, ui: &mut Ui) {
    ui.label(format!("{}", record.datetime_created.date_naive()));
}

fn datetime(record: &Transaction) -> &DateTime<Local> {
    record.datetime()
}

fn d_datetime(record: &Transaction, ui: &mut Ui) {
    ui.label(format!("{}", record.datetime().date_naive()));
}

fn uuid(record: &Transaction) -> &Uuid {
    &record.uuid
}

fn d_uuid(record: &Transaction, ui: &mut Ui) {
    ui.label(format!("{}", *record.uuid));
}

fn amount(record: &Transaction) -> &i32 {
    &record.movement.amount
}

fn d_amount(record: &Transaction, ui: &mut Ui) {
    ui.label(record.amount().to_string());
}

//fn d_description(record: &Transaction, ui: &mut Ui) {
//    ui.label(
//        record
//            .description()
//            .unwrap_or(" -- es gibt keine Beschreibung -- "),
//    );
//}
//
//fn d_tags(record: &Transaction, ui: &mut Ui) {
//    ui.label(format!("{:?}", record.tags()));
//}
//
//fn origin(record: &Transaction) -> &String {
//    record.origin()
//}
//
//fn d_origin(record: &Transaction, ui: &mut Ui) {
//    ui.label(record.origin().to_string());
//}
