use chrono::{DateTime, Local};
use data_communicator::buffered::communicator::Communicator;
use egui::Ui;
use uuid::Uuid;

use crate::{components::soft_button::soft_button, model::records::ExpenseRecord};

pub(super) struct TableColumns {
    datetime_created: TableColumn<DateTime<Local>>,
    uuid: TableColumn<Uuid>,
    amount: TableColumn<isize>,
    description: TableColumn<String>,
    tags: TableColumn<Vec<String>>,
    datetime: TableColumn<DateTime<Local>>,
    origin: TableColumn<String>,
}

impl TableColumns {
    pub(super) fn toggles(&mut self, ui: &mut Ui) {
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
    pub(super) fn header(&self, records: &mut Communicator<Uuid, ExpenseRecord>, ui: &mut Ui) {
        self.datetime_created.display_header(records, ui);
        self.datetime.display_header(records, ui);
        self.uuid.display_header(records, ui);
        self.amount.display_header(records, ui);
        self.description.display_header(records, ui);
        self.tags.display_header(records, ui);
        self.origin.display_header(records, ui);
    }
    pub(super) fn row(&self, record: &ExpenseRecord, ui: &mut Ui) {
        self.datetime_created.display_value(record, ui);
        self.datetime.display_value(record, ui);
        self.uuid.display_value(record, ui);
        self.amount.display_value(record, ui);
        self.description.display_value(record, ui);
        self.tags.display_value(record, ui);
        self.origin.display_value(record, ui);
    }
}

impl Default for TableColumns {
    fn default() -> Self {
        Self {
            datetime_created: TableColumn::inactive("datetime created", d_created),
            uuid: TableColumn::active("uuid", d_uuid).extract_fn(uuid),
            amount: TableColumn::active("amount", d_amount).extract_fn(amount),
            description: TableColumn::active("description", d_description),
            tags: TableColumn::active("tags", d_tags),
            datetime: TableColumn::active("datetime", d_datetime).extract_fn(datetime),
            origin: TableColumn::active("origin", d_origin).extract_fn(origin),
        }
    }
}

struct TableColumn<T> {
    name: &'static str,
    active: bool,
    search_value: Option<T>,
    extract_fn: Option<fn(&ExpenseRecord) -> &T>,
    display: fn(&ExpenseRecord, &mut Ui),
}

impl<T> TableColumn<T>
where
    T: Ord + 'static,
{
    fn active(name: &'static str, display: fn(&ExpenseRecord, &mut Ui)) -> Self {
        Self {
            name,
            active: true,
            search_value: None,
            extract_fn: None,
            display,
        }
    }

    fn inactive(name: &'static str, display: fn(&ExpenseRecord, &mut Ui)) -> Self {
        Self {
            name,
            active: false,
            search_value: None,
            extract_fn: None,
            display,
        }
    }

    fn extract_fn(mut self, func: fn(&ExpenseRecord) -> &T) -> Self {
        self.extract_fn = Some(func);
        self
    }

    fn toggle(&mut self, ui: &mut Ui) {
        ui.checkbox(&mut self.active, self.name);
    }

    fn display_header(&self, records: &mut Communicator<Uuid, ExpenseRecord>, ui: &mut Ui) {
        if !self.active {
            return;
        }

        if let Some(extract_fn) = self.extract_fn {
            let response = soft_button(format!("{}_sorting", self.name), self.name, ui);
            if response.double_clicked() {
                records.sort(move |a, b| extract_fn(b).cmp(extract_fn(a)));
            } else if response.clicked() {
                records.sort(move |a, b| extract_fn(a).cmp(extract_fn(b)));
            }
            if let Some(size) = response.intrinsic_size {
                ui.set_width(size.x);
            }
        } else {
            ui.label(self.name);
        }
    }

    fn display_value(&self, record: &ExpenseRecord, ui: &mut Ui) {
        if !self.active {
            return;
        }
        (self.display)(record, ui);
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
