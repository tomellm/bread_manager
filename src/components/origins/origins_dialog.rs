use std::mem;

use egui::{Id, Modal, ScrollArea, Ui};
use hermes::{
    carrier::execute::ImplExecuteCarrier,
    container::{data::ImplData, manual},
};

use crate::{
    apps::utils::text, components::clamp_str,
    db::query::origins_query::OriginsQuery, model::origins::Origin,
};

#[derive(Default)]
pub struct SelectOriginState {
    is_open: bool,
    create_name: String,
    create_description: String,
}

impl SelectOriginState {
    fn take_create_vars(&mut self) -> (String, String) {
        (
            mem::take(&mut self.create_name),
            mem::take(&mut self.create_description),
        )
    }

    fn create_are_set(&self) -> bool {
        !self.create_name.is_empty() && !self.create_description.is_empty()
    }
}

pub trait SelectOriginDialog {
    fn select_origin_dialog(
        &mut self,
        state: &mut SelectOriginState,
        selected_origin: &mut Option<Origin>,
        origins: &mut manual::Container<Origin>,
    );
}

impl SelectOriginDialog for Ui {
    fn select_origin_dialog(
        &mut self,
        state: &mut SelectOriginState,
        selected_origin: &mut Option<Origin>,
        origins: &mut manual::Container<Origin>,
    ) {
        self.add_enabled_ui(!state.is_open, |ui| {
            if ui.button("set origin").clicked() {
                state.is_open = true;
            }
        });

        if state.is_open {
            Modal::new(Id::new("Modal for Creating Origins")).show(
                self.ctx(),
                |ui| {
                    if ui.button("close").clicked() {
                        state.is_open = false;
                    }
                    ui.heading("Create Origin");
                    text(ui, &mut state.create_name);
                    ui.add(egui::TextEdit::multiline(
                        &mut state.create_description,
                    ));

                    ui.add_enabled_ui(state.create_are_set(), |ui| {
                        if ui.button("save").clicked() {
                            let vars = state.take_create_vars();
                            origins.insert(Origin::init(vars.0, vars.1));
                        }
                    });
                    ui.separator();
                    ui.heading("Origins");
                    origins_scrollarea(state, selected_origin, origins, ui);
                },
            );
        }
    }
}

fn origins_scrollarea(
    state: &mut SelectOriginState,
    selected_origin: &mut Option<Origin>,
    origins: &mut manual::Container<Origin>,
    ui: &mut Ui,
) {
    if origins.data().is_empty() {
        ui.label(NO_ORIGINS_EMPTY_TEXT);
    }

    let mut actor = origins.actor();

    ScrollArea::new([true, true]).show_rows(
        ui,
        50.,
        origins.data().len(),
        |ui, row_rage| {
            row_rage.for_each(|index| {
                if let Some(origin) = origins.data().get(index) {
                    ui.horizontal(|ui| {
                        ui.label(&origin.name);
                        ui.label(clamp_str(&origin.description, 20));
                        if ui.button("select").clicked() {
                            selected_origin.replace(origin.clone());
                            state.is_open = false;
                        }
                        if ui.button("x").clicked() {
                            actor.execute(
                                manual::Container::<Origin>::delete_query(
                                    origin.uuid,
                                ),
                            );

                            if selected_origin
                                .as_ref()
                                .map(|o| o.eq(origin))
                                .unwrap_or_default()
                            {
                                let _ = selected_origin.take();
                            }
                        }
                    });
                }
            });
        },
    );
}

const NO_ORIGINS_EMPTY_TEXT: &str = r#"
No Origins exist as of yet in the Database. Use the above form to create a new origin.
"#;
