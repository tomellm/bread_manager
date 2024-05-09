use egui::Grid;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{model::profiles::Profile, utils::communicator::Communicator};

use self::create_profile::CreateProfile;

mod create_profile;

pub struct Profiles {
    create_profile: CreateProfile,
    profiles_communicator: Communicator<Uuid, Profile>,
}

impl eframe::App for Profiles {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Profiles");

            let mut delete_action = self.profiles_communicator.delete_action();
            let profiles = self.profiles_communicator.view();

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
                        ui.label(format!("{}", profile.profile_width));
                        ui.group(|ui| {
                            if ui.button("delete").clicked() {
                                delete_action(*key);
                            }
                        });
                        ui.end_row();
                    }
                });
            }
            ui.separator();
            self.create_profile.ui(ctx, ui)
        });
    }
}

impl Profiles {
    pub fn new(
        reciver: mpsc::Receiver<egui::DroppedFile>,
        profiles_communicator: Communicator<Uuid, Profile>,
    ) -> Self {
        Self {
            create_profile: CreateProfile::new(reciver, profiles_communicator.clone()),
            profiles_communicator,
        }
    }
}
