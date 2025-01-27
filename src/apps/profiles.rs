mod create_profile;
mod parser;

use eframe::App;
use egui::{Grid, ScrollArea};
use egui_light_states::UiStates;
use hermes::{
    carrier::{execute::ImplExecuteCarrier, query::ImplQueryCarrier},
    container::{data::ImplData, projecting::ProjectingContainer},
    factory::Factory,
    ToActiveModel,
};
use sea_orm::{EntityOrSelect, EntityTrait};
use tokio::sync::mpsc;

use crate::{db::profiles::DbProfile, model::profiles::Profile};

use self::create_profile::CreateProfile;

pub struct Profiles {
    create_profile: CreateProfile,
    profiles: ProjectingContainer<Profile, DbProfile>,
    ui_states: UiStates,
}

impl App for Profiles {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.profiles.state_update();

        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::both().show(ui, |ui| {
                ui.heading("Profiles");

                let delete_action = self.profiles.action();
                let profiles = self.profiles.data();

                if profiles.is_empty() {
                    ui.label("there are currently no profiles to be shown");
                } else {
                    Grid::new("all profiles view").show(ui, |ui| {
                        ui.label("uuid");
                        ui.label("name");
                        ui.label("top margin");
                        ui.label("btm margin");
                        ui.label("delimiter");
                        ui.label("total width");
                        ui.label("default tags");
                        ui.label("edit button");
                        ui.end_row();
                        for profile in profiles {
                            ui.label(profile.uuid.to_string());
                            ui.label(profile.name.clone());
                            ui.label(format!("{}", profile.margins.0));
                            ui.label(format!("{}", profile.margins.1));
                            ui.label(profile.delimiter.to_string());
                            ui.label(format!("{}", profile.width));
                            ui.group(|ui| {
                                for default_tag in &profile.default_tags {
                                    ui.label(default_tag);
                                }
                            });
                            ui.group(|ui| {
                                if ui.button("edit").clicked() {
                                    self.create_profile.edit(profile);
                                }
                                if ui.button("delete").clicked() {
                                    delete_action(DbProfile::delete(profile.dml_clone()));
                                    //set_promise(delete_action(*key).into());
                                }
                                //self.ui_states
                                //    .default_promise_await(format!(
                                //        "delete_action_{}",
                                //        profile.uuid
                                //    ))
                                //    .init_ui(|ui, set_promise| {
                                //    })
                                //    .show(ui);
                            });
                            ui.end_row();
                        }
                    });
                }
                ui.separator();
                self.create_profile.ui_update(ui);
            });
        });
    }
}

impl Profiles {
    pub fn init(
        reciver: mpsc::Receiver<egui::DroppedFile>,
        factory: Factory,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            let mut profiles = factory.builder().projector();
            profiles.query(DbProfile::find().select());
            Self {
                create_profile: CreateProfile::new(reciver, factory),
                profiles,
                ui_states: UiStates::default(),
            }
        }
    }
}
