use crate::model::profiles::{ExpenseDate, ExpenseDateTime, ExpenseTime, ParsableWrapper};
use egui::{Layout, Ui};

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

pub fn blank_option_display<T: std::fmt::Display>(val: Option<&T>) -> String {
    val.map_or_else(|| String::from(""), |val| format!("{val}"))
}

pub fn window_size(ui: &egui::Ui) -> WindowSize {
    WindowSize::from_num(ui.available_width().floor() as usize)
}
