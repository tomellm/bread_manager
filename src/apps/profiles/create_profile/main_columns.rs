use std::hash::Hash;

use egui::Ui;

use crate::{
    apps::utils::{drag_int, option_display, text},
    model::profiles::{
        builder::IntermediateProfileState,
        columns::{
            money::{Expense, Income, Movement, NumberFormat, PosExpense},
            time::{ExpenseDate, ExpenseDateTime, ExpenseTime},
            DateTimeColumn, ExpenseColumn,
        },
    },
};

pub(super) fn expense_col(
    ui: &mut Ui,
    IntermediateProfileState { expense_col, .. }: &mut IntermediateProfileState,
) {
    ui.group(|ui| {
        ui.vertical(|ui| expense_col_selection(ui, expense_col));
    });
}

pub(super) fn datetime_col(
    ui: &mut Ui,
    IntermediateProfileState { datetime_col, .. }: &mut IntermediateProfileState,
) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            datetime_col_selection(ui, datetime_col);
        });
    });
}

fn expense_col_selection(ui: &mut Ui, expense_col: &mut Option<ExpenseColumn>) {
    ui.label("Select the main expense column/s");
    ui.horizontal(|ui| {
        if let Some(expense) = expense_col {
            match expense {
                ExpenseColumn::Split((pos1, Income(inc_format)), (pos2, Expense(exp_format))) => {
                    ui.vertical(|ui| {
                        drag_int(ui, pos1);
                        number_format_combobox("left_format_combobox", inc_format, ui);
                    });
                    ui.vertical(|ui| {
                        drag_int(ui, pos2);
                        number_format_combobox("right_format_combobox", exp_format, ui);
                    });
                }
                ExpenseColumn::Combined(pos, Movement(format)) | ExpenseColumn::OnlyExpense(pos, PosExpense(format)) => {
                    drag_int(ui, pos);
                    number_format_combobox("format_combobox", format, ui)
                }
            }
        }
    });
    egui::ComboBox::from_label("expense")
        .selected_text(
            expense_col
                .as_ref()
                .map_or_else(|| String::from("Nothing"), |v| format!("{v}")),
        )
        .show_ui(ui, |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
            ui.set_min_width(60.0);
            ui.selectable_value(expense_col, None, "not yet");
            ui.selectable_value(expense_col, Some(ExpenseColumn::defaul_split()), "Split");
            ui.selectable_value(
                expense_col,
                Some(ExpenseColumn::default_combined()),
                "Combined",
            );
            ui.selectable_value(
                expense_col,
                Some(ExpenseColumn::default_only_expense()),
                "Only Expense",
            );
        });
}

fn datetime_col_selection(ui: &mut Ui, datetime_col: &mut Option<DateTimeColumn>) {
    ui.horizontal(|ui| {
        if let Some(datetime) = datetime_col {
            match datetime {
                DateTimeColumn::DateAndTime(
                    (pos1, ExpenseDate(format1)),
                    (pos2, ExpenseTime(format2)),
                ) => {
                    ui.vertical(|ui| {
                        drag_int(ui, pos1);
                        drag_int(ui, pos2);
                    });
                    ui.vertical(|ui| {
                        text(ui, format1);
                        text(ui, format2);
                    });
                }
                DateTimeColumn::DateTime(pos, ExpenseDateTime(format))
                | DateTimeColumn::Date(pos, ExpenseDate(format)) => {
                    drag_int(ui, pos);
                    text(ui, format);
                }
            }
        }
    });
    egui::ComboBox::from_label("datetime")
        .selected_text(option_display(datetime_col.as_ref()))
        .show_ui(ui, |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
            ui.set_min_width(60.0);
            ui.selectable_value(datetime_col, None, "not yet");
            ui.selectable_value(datetime_col, Some(DateTimeColumn::new_date()), "Date");
            ui.selectable_value(
                datetime_col,
                Some(DateTimeColumn::new_datetime()),
                "DateTime",
            );
            ui.selectable_value(
                datetime_col,
                Some(DateTimeColumn::new_date_time()),
                "DateAndTime",
            );
        });
}

fn number_format_combobox(id_salt: impl Hash, format: &mut NumberFormat, ui: &mut Ui) {
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(format.to_string())
        .show_ui(ui, |ui| {
            ui.selectable_value(format, NumberFormat::European, "European");
            ui.selectable_value(format, NumberFormat::American, "American");
        });
}
