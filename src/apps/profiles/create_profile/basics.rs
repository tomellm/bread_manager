use egui::Ui;
use hermes::container::manual;

use crate::{
    apps::utils::{drag_int, single_char, text},
    components::{
        origins::origins_dialog::{SelectOriginDialog, SelectOriginState},
        tags::tags_dialog::{SelectTagsDialog, SelectTagsState},
    },
    model::{
        origins::Origin, profiles::builder::IntermediateProfileState, tags::Tag,
    },
};

pub(super) fn name(
    ui: &mut Ui,
    IntermediateProfileState { name, .. }: &mut IntermediateProfileState,
) {
    ui.horizontal(|ui| {
        ui.label("Name: ");
        text(ui, name);
    });
}

pub(super) fn origin(
    ui: &mut Ui,
    origins_state: &mut SelectOriginState,
    state: &mut Option<Origin>,
    origins: &mut manual::Container<Origin>,
) {
    ui.horizontal(|ui| {
        ui.label("Origin: ");
        match state {
            Some(origin) => ui.label(&origin.name),
            None => ui.label("use dialog to select an origin"),
        };
        ui.select_origin_dialog(origins_state, state, origins);
    });
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
    tags_state: &mut SelectTagsState,
    state: &mut Vec<Tag>,
    tags: &mut manual::Container<Tag>,
) {
    ui.horizontal(|ui| {
        ui.label("Tags: ");
        match &state.is_empty() {
            false => {
                state.iter().for_each(|tag| {
                    ui.label(&tag.tag);
                });
            }
            true => {
                ui.label("use dialog to add tags");
            }
        };
        ui.select_tags_dialog(tags_state, state, tags);
    });
}
