use egui::{Layout, Ui};

use crate::{
    apps::utils::{drag_int, text},
    model::profiles::{
        builder::IntermediateProfileState,
        columns::{
            other::{Description, Other},
            time::{ExpenseDate, ExpenseDateTime, ExpenseTime},
            ParsableWrapper,
        },
    },
};

pub(super) fn other_cols(
    ui: &mut Ui,
    IntermediateProfileState { other_cols, .. }: &mut IntermediateProfileState,
) {
    ui.horizontal(|ui| {
        if ui.button("other").clicked() {
            other_cols.push((0, ParsableWrapper::income()));
        }
    });
    ui.add_space(10.);
    ui.horizontal_wrapped(|ui| {
        other_cols.retain_mut(|(ref mut col_pos, ref mut col_type)| {
            let mut retain: bool = true;
            ui.add_sized([175., 175.], |ui: &mut Ui| {
                ui.group(|ui| {
                    ui.vertical_centered(|ui| {
                        egui::ComboBox::from_id_salt(format!("other col {col_pos}"))
                            .selected_text(format!("{col_type}"))
                            .show_ui(ui, |ui| {
                                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                                ui.selectable_value(col_type, ParsableWrapper::income(), "Income");
                                ui.selectable_value(
                                    col_type,
                                    ParsableWrapper::expense(),
                                    "Expense",
                                );
                                ui.selectable_value(
                                    col_type,
                                    ParsableWrapper::posexpense(),
                                    "PosExpense",
                                );
                                ui.selectable_value(
                                    col_type,
                                    ParsableWrapper::movement(),
                                    "Movement",
                                );
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
                                    ParsableWrapper::Description(Description(s))
                                    | ParsableWrapper::Other(Other(s))
                                    | ParsableWrapper::ExpenseDateTime(ExpenseDateTime(s))
                                    | ParsableWrapper::ExpenseDate(ExpenseDate(s))
                                    | ParsableWrapper::ExpenseTime(ExpenseTime(s)) => text(ui, s),
                                    _ => (),
                                }
                            })
                            .response
                        });
                        ui.separator();
                        ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                            if ui.button("remove").clicked() {
                                retain = false;
                            }
                        });
                    });
                })
                .response
            });
            retain
        });
    });
}
