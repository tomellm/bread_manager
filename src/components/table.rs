use data_communicator::buffered::{communicator::Communicator, ValueBounds};
use egui::Ui;
use uuid::Uuid;

use super::soft_button::soft_button;

pub(crate) struct TableColumn<T, P> {
    name: &'static str,
    active: bool,
    search_value: Option<P>,
    extract_fn: Option<fn(&T) -> &P>,
    display: fn(&T, &mut Ui),
}

impl<T, P> TableColumn<T, P>
where
    T: ValueBounds<Uuid>,
    P: Ord + 'static,
{
    pub(crate) fn active(name: &'static str, display: fn(&T, &mut Ui)) -> Self {
        Self {
            name,
            active: true,
            search_value: None,
            extract_fn: None,
            display,
        }
    }

    pub(crate) fn inactive(name: &'static str, display: fn(&T, &mut Ui)) -> Self {
        Self {
            name,
            active: false,
            search_value: None,
            extract_fn: None,
            display,
        }
    }

    pub(crate) fn extract_fn(mut self, func: fn(&T) -> &P) -> Self {
        self.extract_fn = Some(func);
        self
    }

    pub(crate) fn toggle(&mut self, ui: &mut Ui) {
        ui.checkbox(&mut self.active, self.name);
    }

    pub fn header(&self, ui: &mut Ui) {
        if self.active {
            ui.label(self.name);
        }
    }

    pub(crate) fn sorting_header(&self, records: &mut Communicator<Uuid, T>, ui: &mut Ui) {
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

    pub(crate) fn display_value(&self, record: &T, ui: &mut Ui) {
        if !self.active {
            return;
        }
        (self.display)(record, ui);
    }
}
