use chrono::{DateTime, Local};
use egui::Ui;
use hermes::container::data::ImplData;
use uuid::Uuid;

use crate::{components::table::TableColumn, model::records::ExpenseRecord};

pub(crate) struct RecordsTable {
    datetime_created: TableColumn<ExpenseRecord, DateTime<Local>>,
    uuid: TableColumn<ExpenseRecord, Uuid>,
    amount: TableColumn<ExpenseRecord, isize>,
    description: TableColumn<ExpenseRecord, String>,
    tags: TableColumn<ExpenseRecord, Vec<String>>,
    datetime: TableColumn<ExpenseRecord, DateTime<Local>>,
    origin: TableColumn<ExpenseRecord, String>,
}

impl RecordsTable {
    pub(crate) fn toggles(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.datetime_created.toggle(ui);
            self.datetime.toggle(ui);
            self.uuid.toggle(ui);
            self.amount.toggle(ui);
            self.description.toggle(ui);
            self.tags.toggle(ui);
            self.origin.toggle(ui);
        });
    }
    pub(crate) fn header(&self, ui: &mut Ui) {
        self.datetime_created.header(ui);
        self.datetime.header(ui);
        self.uuid.header(ui);
        self.amount.header(ui);
        self.description.header(ui);
        self.tags.header(ui);
        self.origin.header(ui);
    }
    pub(crate) fn sorting_header(
        &self,
        records: &mut impl ImplData<ExpenseRecord>,
        ui: &mut Ui,
    ) {
        self.datetime_created.sorting_header(records, ui);
        self.datetime.sorting_header(records, ui);
        self.uuid.sorting_header(records, ui);
        self.amount.sorting_header(records, ui);
        self.description.sorting_header(records, ui);
        self.tags.sorting_header(records, ui);
        self.origin.sorting_header(records, ui);
    }
    pub(crate) fn row(&self, record: &ExpenseRecord, ui: &mut Ui) {
        self.datetime_created.display_value(record, ui);
        self.datetime.display_value(record, ui);
        self.uuid.display_value(record, ui);
        self.amount.display_value(record, ui);
        self.description.display_value(record, ui);
        self.tags.display_value(record, ui);
        self.origin.display_value(record, ui);
    }
    pub(crate) fn show(&self, records: &Vec<ExpenseRecord>, ui: &mut Ui) {
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
        records: &mut impl ImplData<ExpenseRecord>,
        filter: impl FnMut(&&ExpenseRecord) -> bool,
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

impl Default for RecordsTable {
    fn default() -> Self {
        Self {
            datetime_created: TableColumn::inactive(
                "datetime created",
                d_created,
            ),
            uuid: TableColumn::active("uuid", d_uuid).extract_fn(uuid),
            amount: TableColumn::active("amount", d_amount).extract_fn(amount),
            description: TableColumn::active("description", d_description),
            tags: TableColumn::active("tags", d_tags),
            datetime: TableColumn::active("datetime", d_datetime)
                .extract_fn(datetime),
            origin: TableColumn::active("origin", d_origin).extract_fn(origin),
        }
    }
}

fn d_created(record: &ExpenseRecord, ui: &mut Ui) {
    ui.label(format!("{}", record.created().date_naive()));
}

fn datetime(record: &ExpenseRecord) -> &DateTime<Local> {
    record.datetime()
}

fn d_datetime(record: &ExpenseRecord, ui: &mut Ui) {
    ui.label(format!("{}", record.datetime().date_naive()));
}

fn uuid(record: &ExpenseRecord) -> &Uuid {
    record.uuid()
}

fn d_uuid(record: &ExpenseRecord, ui: &mut Ui) {
    ui.label(format!("{}", record.uuid().0));
}

fn amount(record: &ExpenseRecord) -> &isize {
    record.amount()
}

fn d_amount(record: &ExpenseRecord, ui: &mut Ui) {
    ui.label(record.formatted_amount());
}

fn d_description(record: &ExpenseRecord, ui: &mut Ui) {
    ui.label(format!("{:?}", record.description()));
}

fn d_tags(record: &ExpenseRecord, ui: &mut Ui) {
    ui.label(format!("{:?}", record.tags()));
}

fn origin(record: &ExpenseRecord) -> &String {
    record.origin()
}

fn d_origin(record: &ExpenseRecord, ui: &mut Ui) {
    ui.label(record.origin().to_string());
}
