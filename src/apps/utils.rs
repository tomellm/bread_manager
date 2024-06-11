use std::{cell::RefCell, rc::Rc};

use crate::model::profiles::{ExpenseDate, ExpenseDateTime, ExpenseTime, ParsableWrapper};
use egui::{Layout, Response, Ui};

pub enum WindowSize {
    Small,
    Medium,
    Large,
}

impl WindowSize {
    pub fn from_num(num: usize) -> Self {
        match num {
            0..=699 => Self::Small,
            700..=1199 => Self::Medium,
            _ => Self::Large,
        }
    }
}

pub fn drag_int(ui: &mut egui::Ui, val: &mut usize) {
    ui.add(egui::DragValue::new(val).speed(0.1).max_decimals(0));
}

pub fn single_char(ui: &mut egui::Ui, val: &mut String) {
    ui.add(egui::TextEdit::singleline(val).char_limit(1));
}

pub fn text(ui: &mut egui::Ui, val: &mut String) {
    ui.add(egui::TextEdit::singleline(val));
}

pub fn option_display<T: std::fmt::Display>(val: Option<&T>) -> String {
    val.map_or_else(|| String::from("Nothing"), |val| format!("{val}"))
}

pub fn window_size(ui: &egui::Ui) -> WindowSize {
    WindowSize::from_num(ui.available_width().floor() as usize)
}

pub fn other_column_editor(
    ui: &mut egui::Ui,
    index: usize,
    col_pos: &mut usize,
    col_type: &mut ParsableWrapper,
    to_delete: &Rc<RefCell<Vec<usize>>>,
) -> Response {
    let response = ui
        .group(|ui| {
            ui.vertical_centered(|ui| {
                egui::ComboBox::from_id_source(format!("other col {index}"))
                    .selected_text(format!("{col_type}"))
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap = Some(true);
                        ui.selectable_value(col_type, ParsableWrapper::income(), "Income");
                        ui.selectable_value(col_type, ParsableWrapper::expense(), "Expense");
                        ui.selectable_value(col_type, ParsableWrapper::posexpense(), "PosExpense");
                        ui.selectable_value(col_type, ParsableWrapper::movement(), "Movement");
                        ui.selectable_value(
                            col_type,
                            ParsableWrapper::expensedatetime(),
                            "ExpenseDatetime",
                        );
                        ui.selectable_value(
                            col_type,
                            ParsableWrapper::expensedate(),
                            "ExpenseDate",
                        );
                        ui.selectable_value(
                            col_type,
                            ParsableWrapper::expensetime(),
                            "ExpenseTime",
                        );
                        ui.selectable_value(
                            col_type,
                            ParsableWrapper::description(),
                            "Description",
                        );
                        ui.selectable_value(col_type, ParsableWrapper::other(), "Other");
                    });
                ui.separator();
                ui.add_sized([160., 100.], |ui: &mut Ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Pos:");
                            drag_int(ui, col_pos);
                        });
                        match col_type {
                            ParsableWrapper::ExpenseDateTime(ExpenseDateTime(f))
                            | ParsableWrapper::ExpenseDate(ExpenseDate(f))
                            | ParsableWrapper::ExpenseTime(ExpenseTime(f)) => text(ui, f),
                            _ => (),
                        }
                    })
                    .response
                });
                ui.separator();
                ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                    if ui.button("remove").clicked() {
                        to_delete.borrow_mut().push(index);
                    }
                });
            });
        })
        .response;

    response
}
