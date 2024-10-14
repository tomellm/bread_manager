use egui::Ui;

use crate::{
    apps::utils::{drag_int, single_char, text}, model::profiles::builder::IntermediateProfileState,
};

pub(super) fn name(
    ui: &mut Ui,
    IntermediateProfileState { name, .. }: &mut IntermediateProfileState,
) {
    text(ui, name);
}

pub(super) fn delimiter(
    ui: &mut Ui,
    IntermediateProfileState { delimiter, .. }: &mut IntermediateProfileState,
) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label("Delimiter");
            single_char(ui, delimiter);
        });
    });
}

pub(super) fn margin_top(
    ui: &mut Ui,
    IntermediateProfileState { margin_top, .. }: &mut IntermediateProfileState,
) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label("Margin Top");
            drag_int(ui, margin_top);
        });
    });
}

pub(super) fn margin_btm(
    ui: &mut Ui,
    IntermediateProfileState { margin_btm, .. }: &mut IntermediateProfileState,
) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label("Margin Bottom");
            drag_int(ui, margin_btm);
        });
    });
}

pub(super) fn default_tags(
    ui: &mut Ui,
    IntermediateProfileState { default_tags, .. }: &mut IntermediateProfileState,
) {
    ui.horizontal(|ui| {
        if ui.button("add default tag").clicked() {
            default_tags.push(String::new());
        }
    });
    ui.add_space(10.);
    ui.horizontal_wrapped(|ui| {
        default_tags.retain_mut(|tag| {
            let mut delete: bool = true;
            ui.add_sized([100., 25.], |ui: &mut Ui| {
                let res = ui.add(egui::TextEdit::singleline(tag));
                if ui.button("remove").clicked() {
                    delete = false;
                }
                res
            });
            delete
        });
    });
}

pub(super) fn origin_name(
    ui: &mut Ui,
    IntermediateProfileState { origin_name, .. }: &mut IntermediateProfileState,
) {
    ui.group(|ui| {
        ui.label("origin name");
        ui.add(egui::TextEdit::singleline(origin_name));
    });
}
