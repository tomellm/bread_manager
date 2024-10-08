use egui::Ui;

use crate::{
    apps::utils::{drag_int, option_display, text},
    model::profiles::{
        DateTimeColumn, ExpenseColumn, ExpenseDate, ExpenseDateTime, ExpenseTime,
        IntermediateProfileState,
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
                ExpenseColumn::Split((pos1, _), (pos2, _)) => {
                    drag_int(ui, pos1);
                    drag_int(ui, pos2);
                }
                ExpenseColumn::Combined(pos, _) | ExpenseColumn::OnlyExpense(pos, _) => {
                    drag_int(ui, pos);
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
            ui.selectable_value(expense_col, Some(ExpenseColumn::split(0, 0)), "Split");
            ui.selectable_value(expense_col, Some(ExpenseColumn::combined(0)), "Combined");
            ui.selectable_value(
                expense_col,
                Some(ExpenseColumn::only_expense(0)),
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
