mod basics;
mod main_columns;
mod other_columns;

use std::sync::Arc;

use basics::{default_tags, delimiter, margin_btm, margin_top, name, origin_name};
use data_communicator::buffered::{change::ChangeResult, communicator::Communicator};
use egui::{Spinner, Ui};
use egui_light_states::{promise_await::{CreatePromiseAwait, DoneResponse}, UiStates};
use lazy_async_promise::ImmediateValuePromise;
use main_columns::{datetime_col, expense_col};
use other_columns::other_cols;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::model::profiles::{
    builder::{IntermediateProfileState, ProfileBuilder},
    Profile,
};

use super::parser::ProfilePreview;

pub struct CreateProfile {
    preview: ProfilePreview,
    profile_builder: Arc<ProfileBuilder>,
    intermediate_profile_state: IntermediateProfileState,
    profiles_communicator: Communicator<Uuid, Profile>,
    ui_states: UiStates,
}

impl CreateProfile {
    pub fn new(
        reciver: mpsc::Receiver<egui::DroppedFile>,
        profiles_communicator: Communicator<Uuid, Profile>,
    ) -> Self {
        Self {
            preview: ProfilePreview::new(reciver),
            profile_builder: Arc::new(ProfileBuilder::default()),
            intermediate_profile_state: IntermediateProfileState::default(),
            profiles_communicator,
            ui_states: UiStates::default(),
        }
    }

    pub fn ui_update(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("Create Profiles:");
            if ui.button("reset").clicked() {
                self.reset();
            }
        });
        self.show_editing_controls(ui);
        if ui.button("update builder").clicked() {
            self.update_builder();
            self.preview.update_parse_test(&self.profile_builder);
        }

        const MIN_PREVIEW_HEIGHT: f32 = 200.;
        let preview_height = if ui.available_height() > MIN_PREVIEW_HEIGHT {
            ui.available_height()
        } else {
            MIN_PREVIEW_HEIGHT
        };
        ui.add_sized([ui.available_width(), preview_height], |ui: &mut Ui| {
            self.preview.profile_preview(ui, &self.profile_builder)
        });

        ui.add_sized([ui.available_width(), 50.], |ui: &mut Ui| {
            ui.vertical_centered_justified(|ui| {
                let save_profile = self.save_profile();
                self.ui_states
                    .promise_await("save profile".into())
                    .init_ui(|ui, set_promise| match save_profile {
                        Ok(save_profile_action) => {
                            if ui.button("save profile").clicked() {
                                set_promise(save_profile_action());
                            };
                        }
                        Err(()) => {
                            ui.label("profile is incorrect and cannot be saved");
                        }
                    })
                .waiting_ui(|ui| {
                    ui.add(Spinner::new());
                })
                .done_ui(|_, _| DoneResponse::<ChangeResult>::Clear)
                    .show(ui);
            })
            .response
        });
    }

    pub fn edit(&mut self, profile: &Profile) {
        self.reset();
        self.intermediate_profile_state = IntermediateProfileState::from_profile(profile);
    }

    fn reset(&mut self) {
        self.preview.reset();
        self.profile_builder = Arc::new(ProfileBuilder::default());
        self.intermediate_profile_state = IntermediateProfileState::default();
    }

    fn show_editing_controls(&mut self, ui: &mut egui::Ui) {
        let CreateProfile {
            intermediate_profile_state: state,
            ..
        } = self;
        name(ui, state);
        ui.horizontal(|ui| {
            delimiter(ui, state);
            margin_top(ui, state);
            margin_btm(ui, state);
        });
        ui.vertical_centered(|ui| {
            default_tags(ui, state);
        });
        ui.add_space(10.);
        ui.separator();
        ui.add_space(10.);
        ui.horizontal(|ui| {
            expense_col(ui, state);
            datetime_col(ui, state);
            origin_name(ui, state)
        });
        ui.add_space(10.);
        ui.separator();
        ui.add_space(10.);
        ui.vertical_centered(|ui| {
            other_cols(ui, state);
        });
        ui.add_space(10.);
    }

    fn update_builder(&mut self) {
        let profile = ProfileBuilder::from_inter_state(&self.intermediate_profile_state);
        if let Ok(profile) = profile {
            self.profile_builder = Arc::new(profile);
        }
    }

    fn save_profile(&mut self) -> Result<impl FnOnce() -> ImmediateValuePromise<ChangeResult>, ()> {
        self.update_builder();
        let profile = (*self.profile_builder).clone().build().map_err(|()| {})?;
        let mut setter = self.profiles_communicator.insert_action();
        Ok(move || ImmediateValuePromise::new(setter(profile)))
    }
}
