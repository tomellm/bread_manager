use data_communicator::buffered::communicator::Communicator;
use eframe::App;
use egui::Grid;
use egui_light_states::{default_promise_await::DefaultCreatePromiseAwait, UiStates};
use lazy_async_promise::ImmediateValuePromise;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::model::profiles::Profile;

use self::create_profile::CreateProfile;

mod create_profile;

pub struct Profiles {
    create_profile: CreateProfile,
    profiles: Communicator<Uuid, Profile>,
    ui_states: UiStates,
}

impl App for Profiles {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Profiles");

            let mut delete_action = self.profiles.delete_action();
            let profiles = self.profiles.data_map();

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
                    ui.label("actions");
                    ui.end_row();
                    for (key, profile) in profiles.iter() {
                        ui.label(profile.uuid.to_string());
                        ui.label(profile.name.clone());
                        ui.label(format!("{}", profile.margins.0));
                        ui.label(format!("{}", profile.margins.1));
                        ui.label(profile.delimiter.to_string());
                        ui.label(format!("{}", profile.width));
                        ui.group(|ui| {
                            self.ui_states
                                .default_promise_await(format!(
                                    "delete action for {}",
                                    key.as_u128()
                                ))
                                .init_ui(|ui, set_promise| {
                                    if ui.button("delete").clicked() {
                                        let future = delete_action(*key);
                                        set_promise(ImmediateValuePromise::new(future));
                                    }
                                });
                            if ui.button("edit").clicked() {
                                self.create_profile.edit(profile);
                            }
                        });
                        ui.end_row();
                    }
                });
            }
            ui.separator();
            self.create_profile.ui(ctx, ui);
        });
    }
}

impl Profiles {
    pub fn new(
        reciver: mpsc::Receiver<egui::DroppedFile>,
        [profile_one, profile_two]: [Communicator<Uuid, Profile>; 2],
    ) -> Self {
        Self {
            create_profile: CreateProfile::new(reciver, profile_one),
            profiles: profile_two,
            ui_states: UiStates::default()
        }
    }
}
