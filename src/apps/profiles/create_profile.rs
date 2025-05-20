mod basics;
mod main_columns;
mod other_columns;

use std::sync::Arc;

use basics::{default_tags, delimiter, margin_btm, margin_top, name, origin};
use egui::Ui;
use hermes::{container::manual, factory::Factory};
use main_columns::{datetime_col, expense_col};
use other_columns::other_cols;
use tokio::sync::mpsc;

use crate::{
    components::{
        origins::origins_dialog::SelectOriginState,
        tags::tags_dialog::SelectTagsState,
    },
    db::query::{
        origins_query::OriginsQuery, profile_query::ProfileQuery,
        tags_query::TagsQuery,
    },
    model::{
        origins::Origin,
        profiles::{
            builder::{CreateProfileBuilder, IntermediateProfileState},
            Profile,
        },
        tags::Tag,
    },
};

use super::parser::ProfilePreview;

pub struct CreateProfile {
    preview: ProfilePreview,
    profile_builder: Arc<CreateProfileBuilder>,
    intermediate_profile_state: IntermediateProfileState,
    profiles: manual::Container<Profile>,
    origins: manual::Container<Origin>,
    tags: manual::Container<Tag>,
    select_origins_state: SelectOriginState,
    select_tags_state: SelectTagsState,
}

impl CreateProfile {
    pub fn new(
        reciver: mpsc::Receiver<egui::DroppedFile>,
        factory: Factory,
    ) -> Self {
        let mut profiles =
            factory.builder().name("create_profile_profiles").manual();
        profiles.stored_query(ProfileQuery::all);

        let mut origins =
            factory.builder().name("create_profile_origins").manual();
        origins.stored_query(OriginsQuery::all);

        let mut tags = factory.builder().name("create_profile_tags").manual();
        tags.stored_query(TagsQuery::all);

        Self {
            preview: ProfilePreview::new(reciver),
            profile_builder: Arc::new(CreateProfileBuilder::default()),
            intermediate_profile_state: IntermediateProfileState::default(),
            profiles,
            select_origins_state: SelectOriginState::default(),
            select_tags_state: SelectTagsState::default(),
            origins,
            tags,
        }
    }

    pub fn ui_update(&mut self, ui: &mut Ui) {
        self.profiles.state_update(true);
        self.origins.state_update(true);
        self.tags.state_update(true);

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
            ui.vertical_centered_justified(|ui| match self.parse_profile() {
                Ok(profile) => {
                    if ui.button("save profile").clicked() {
                        self.profiles.insert(profile);
                    };
                }
                Err(()) => {
                    ui.label("profile is incorrect and cannot be saved");
                }
            })
            .response
        });
    }

    pub fn edit(&mut self, profile: &Profile) {
        self.reset();
        self.intermediate_profile_state =
            IntermediateProfileState::from_profile(profile);
    }

    fn reset(&mut self) {
        self.preview.reset();
        self.profile_builder = Arc::new(CreateProfileBuilder::default());
        self.intermediate_profile_state = IntermediateProfileState::default();
    }

    fn show_editing_controls(&mut self, ui: &mut egui::Ui) {
        let CreateProfile {
            intermediate_profile_state: state,
            ..
        } = self;
        ui.horizontal(|ui| {
            name(ui, state);
            origin(
                ui,
                &mut self.select_origins_state,
                &mut state.origin,
                &mut self.origins,
            );
        });
        ui.horizontal(|ui| {
            delimiter(ui, state);
            margin_top(ui, state);
            margin_btm(ui, state);
        });
        ui.vertical_centered(|ui| {
            default_tags(
                ui,
                &mut self.select_tags_state,
                &mut state.default_tags,
                &mut self.tags,
            );
        });
        ui.add_space(10.);
        ui.separator();
        ui.add_space(10.);
        ui.horizontal(|ui| {
            expense_col(ui, state);
            datetime_col(ui, state);
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
        let profile = CreateProfileBuilder::from_inter_state(
            &self.intermediate_profile_state,
        );
        if let Ok(profile) = profile {
            self.profile_builder = Arc::new(profile);
        }
    }

    fn parse_profile(&mut self) -> Result<Profile, ()> {
        self.update_builder();
        (*self.profile_builder).clone().build()
    }
}
