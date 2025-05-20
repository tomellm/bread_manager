use std::mem;

use egui::{Id, Modal, ScrollArea, Ui};
use hermes::{
    carrier::execute::ImplExecuteCarrier,
    container::{data::ImplData, manual},
};

use crate::{
    apps::utils::text,
    components::clamp_str,
    db::query::tags_query::TagsQuery,
    model::tags::{Tag, TagUuid},
};

#[derive(Default)]
pub struct SelectTagsState {
    is_open: bool,
    create_name: String,
    create_description: String,
}

impl SelectTagsState {
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

pub trait SelectTagsDialog {
    fn select_tags_dialog(
        &mut self,
        state: &mut SelectTagsState,
        selected_tags: &mut Vec<Tag>,
        tags: &mut manual::Container<Tag>,
    );
}

impl SelectTagsDialog for Ui {
    fn select_tags_dialog(
        &mut self,
        state: &mut SelectTagsState,
        selected_tags: &mut Vec<Tag>,
        tags: &mut manual::Container<Tag>,
    ) {
        self.add_enabled_ui(!state.is_open, |ui| {
            if ui.button("set tags").clicked() {
                state.is_open = true;
            }
        });
        if state.is_open {
            Modal::new(Id::new("Modal for Creating Tags")).show(
                self.ctx(),
                |ui| {
                    if ui.button("close").clicked() {
                        state.is_open = false;
                    }
                    ui.heading("Create Tag");
                    text(ui, &mut state.create_name);
                    ui.add(egui::TextEdit::multiline(
                        &mut state.create_description,
                    ));
                    ui.add_enabled_ui(state.create_are_set(), |ui| {
                        if ui.button("save").clicked() {
                            let vars = state.take_create_vars();
                            tags.insert(Tag::init(vars.0, vars.1));
                        }
                    });
                    ui.separator();
                    ui.heading("Origins");
                    tags_scrollarea(state, selected_tags, tags, ui);
                },
            );
        }
    }
}

fn tags_scrollarea(
    state: &mut SelectTagsState,
    selected_tags: &mut Vec<Tag>,
    tags: &mut manual::Container<Tag>,
    ui: &mut Ui,
) {
    if tags.data().is_empty() {
        ui.label(NO_TAGS_EMPTY_TEXT);
    }

    let mut actor = tags.actor();

    ScrollArea::new([true, true]).show_rows(
        ui,
        50.,
        tags.data().len(),
        |ui, row_rage| {
            row_rage.for_each(|index| {
                if let Some(tag) = tags.data().get(index) {
                    ui.horizontal(|ui| {
                        ui.label(&tag.tag);
                        ui.label(clamp_str(&tag.description, 20));
                        if ui.button("add").clicked() {
                            selected_tags.push(tag.clone());
                        }
                        ui.add_enabled_ui(selected_tags.contains(tag), |ui| {
                            if ui.button("remove").clicked() {
                                let _ = selected_tags
                                    .extract_if(.., |el| el.uuid.eq(&tag.uuid));
                            }
                        });
                        if ui.button("x").clicked() {
                            actor.execute(
                                manual::Container::<Tag>::delete_query(
                                    tag.uuid,
                                ),
                            );
                            let _ = selected_tags
                                .extract_if(.., |el| el.uuid.eq(&tag.uuid));
                        }
                    });
                }
            });
        },
    );
}

const NO_TAGS_EMPTY_TEXT: &str = r#"
No Tags exist as of yet in the Database. Use the above form to create a new Tag.
"#;
