mod create_profile;
mod parser;

use data_communicator::buffered::{communicator::Communicator, query::QueryType};
use eframe::App;
use egui::{Grid, ScrollArea};
use egui_light_states::{default_promise_await::DefaultCreatePromiseAwait, UiStates};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::model::profiles::Profile;

use self::create_profile::CreateProfile;

pub struct Profiles {
    create_profile: CreateProfile,
    profiles: Communicator<Uuid, Profile>,
    ui_states: UiStates,
}

impl App for Profiles {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.profiles.state_update();

        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::both()
                .show(ui, |ui| {
                    ui.heading("Profiles");

                    let mut delete_action = self.profiles.delete_action();
                    let profiles = self.profiles.data.map();

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
                            for (key, profile) in profiles.iter() {
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
                                    self.ui_states
                                        .default_promise_await(format!("delete_action_{key}"))
                                        .init_ui(|ui, set_promise| {
                                            if ui.button("delete").clicked() {
                                                set_promise(delete_action(*key).into());
                                            }
                                        })
                                    .show(ui);
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
        [profile_one, profile_two]: [Communicator<Uuid, Profile>; 2],
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            let _ = profile_one.query(QueryType::All).await;
            let _ = profile_two.query(QueryType::All).await;
            Self {
                create_profile: CreateProfile::new(reciver, profile_one),
                profiles: profile_two,
                ui_states: UiStates::default(),
            }
        }
    }
}
