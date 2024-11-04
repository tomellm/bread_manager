use std::fs;

use diesel::{QueryDsl, SelectableHelper, SqliteConnection};
use egui::{
    popup_below_widget, Color32, ComboBox, DroppedFile, Grid, Id, Label, PopupCloseBehavior,
    RichText, Ui, Widget,
};
use hermes::{container::projecting::ProjectingContainer, factory::Factory};
use num_traits::Zero;
use tokio::sync::mpsc;
use tracing::info;
use uuid::Uuid;

use crate::{db::{profiles::PROFILES_FROM_DB_FN, records::RECORDS_FROM_DB_FN}, schema::profiles::dsl as prof_dsl};

use crate::{apps::DbConn, db::profiles::DbProfile, model::profiles::Profile};

pub(super) struct FilesToParse {
    reciver: mpsc::Receiver<DroppedFile>,
    profiles: ProjectingContainer<Profile, DbProfile, DbConn>,
    files: Vec<FileToParse>,
}

impl FilesToParse {
    pub(super) fn init(
        reciver: mpsc::Receiver<DroppedFile>,
        factory: &Factory<DbConn>,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        let mut profiles = factory.builder().projector_arc(PROFILES_FROM_DB_FN.clone());
        async move {
            profiles.query(|| prof_dsl::profiles.select(DbProfile::as_select()));
            Self {
                reciver,
                profiles,
                files: vec![],
            }
        }
    }

    pub(super) fn files_to_parse_list(&mut self, ui: &mut Ui) {
        self.profiles.state_update();
        self.recive_files();

        if !self.files.is_empty() {
            egui::Grid::new("uploaded files table").show(ui, |ui| {
                ui.label("name");
                ui.label("path");
                ui.label("profile");
                ui.label("margin cutoff");
                ui.label("remove");
                ui.end_row();
                self.files.retain_mut(|file_to_parse| {
                    file_to_parse.update_internal_values();

                    ui.label(file_to_parse.file.name.clone());
                    Self::file_path(file_to_parse, ui);
                    Self::profile_select(file_to_parse, self.profiles.values(), ui);
                    Self::margin_cutoff(file_to_parse, ui);
                    Self::remove_button(ui)
                });
            });
        } else {
            ui.vertical_centered_justified(|ui| {
                ui.add_space(30.);
                ui.label(DROPPED_FILES_EMPTY_TEXT);
                ui.add_space(30.);
            });
        }
    }

    fn file_path(file: &FileToParse, ui: &mut Ui) {
        if let Some(path) = file.file.path.as_ref() {
            ui.label(path.file_name().unwrap().to_str().unwrap())
                .on_hover_text(path.to_str().unwrap());
        } else {
            ui.label("..no path..");
        }
    }

    fn profile_select(file: &mut FileToParse, profiles: &Vec<Profile>, ui: &mut Ui) {
        ComboBox::from_id_salt(format!("select_profile_{:?}", file.uuid))
            .selected_text({
                file.profile
                    .clone()
                    .map_or(String::from("select profile"), |p| p.name)
            })
            .show_ui(ui, |ui| {
                for profile in profiles.iter() {
                    ui.selectable_value(
                        &mut file.profile,
                        Some(profile.clone()),
                        profile.name.clone(),
                    );
                }
            });
    }

    fn margin_cutoff(file: &FileToParse, ui: &mut Ui) {
        let Some(profile) = &file.profile else {
            ui.label("select a profile");
            return;
        };

        let margin_show_response = ui.button("show");
        let popup_id = Id::new(format!("popup_id_{}", file.uuid));
        if margin_show_response.clicked() {
            ui.memory_mut(|mem| mem.toggle_popup(popup_id));
        }
        popup_below_widget(
            ui,
            popup_id,
            &margin_show_response,
            PopupCloseBehavior::CloseOnClickOutside,
            |ui| {
                if !file.cut_off_margins.is_set() {
                    ui.label("Nothing will be cut off.");
                    return;
                }

                ui.label(format!(
                    "Cut off margin for top is: {} and for the bottom {}.",
                    profile.margins.0, profile.margins.1
                ));
                ui.label("Note that only the brigther text will be removed not the dark one");
                ui.separator();

                let luminance = 70;

                Grid::new(format!("cut_off_margis_grid_{}", file.uuid)).show(ui, |ui| {
                    if let Some(top) = file.cut_off_margins.top.as_ref() {
                        for (index, row) in top[0..(top.len() - 1)].iter().enumerate() {
                            ui.label((index + 1).to_string());
                            for el in row {
                                ui.label(el);
                            }
                            ui.end_row();
                        }

                        ui.label("");
                        for el in top.last().unwrap() {
                            Label::new(RichText::new(el).color(Color32::from_gray(luminance)))
                                .ui(ui);
                        }
                        ui.end_row();
                    }

                    ui.label("");
                    for _ in 0..file.cut_off_margins.width().unwrap() {
                        Label::new(RichText::new("...").color(Color32::from_gray(luminance)))
                            .ui(ui);
                    }
                    ui.end_row();

                    if let Some(bottom) = file.cut_off_margins.bottom.as_ref() {
                        ui.label("");
                        for el in bottom.first().unwrap() {
                            Label::new(RichText::new(el).color(Color32::from_gray(luminance)))
                                .ui(ui);
                        }
                        ui.end_row();

                        for (index, row) in bottom[1..bottom.len()].iter().enumerate() {
                            ui.label((profile.margins.1 - index).to_string());
                            for el in row {
                                ui.label(el);
                            }
                            ui.end_row();
                        }
                    }
                });
            },
        );
    }

    fn remove_button(ui: &mut Ui) -> bool {
        if ui.button("remove").clicked() {
            ui.end_row();
            false
        } else {
            ui.end_row();
            true
        }
    }

    pub fn extract_ready_files(&mut self) -> impl Iterator<Item = FileToParse> + '_ {
        self.files.extract_if(|f| f.profile.is_some())
    }

    pub fn recive_files(&mut self) {
        while let Ok(file) = self.reciver.try_recv() {
            info!(
                msg = "Recived dropped file, adding to list",
                file = format!("{file:?}")
            );
            self.files.push(file.into());
        }
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

#[derive(Clone, Debug)]
pub(super) struct FileToParse {
    uuid: Uuid,
    pub(super) file: DroppedFile,
    pub(super) profile: Option<Profile>,
    profile_name: Option<String>,
    cut_off_margins: CutOffMargins,
}

impl FileToParse {
    pub fn update_internal_values(&mut self) {
        let Some(profile) = &mut self.profile else {
            return;
        };

        let reclac_margins = if let Some(profile_name) = &self.profile_name {
            !profile.name.eq(profile_name)
        } else {
            true
        };

        if reclac_margins {
            self.cut_off_margins.clear();

            let _ = self.profile_name.insert(profile.name.clone());
            let path = self.file.path.as_ref().unwrap();
            let str = fs::read_to_string(path).unwrap();
            let len = str.lines().count();

            for (index, line) in str.lines().enumerate() {
                if !profile.margins.0.is_zero() && index < profile.margins.0 + 1 {
                    self.cut_off_margins.push_top(line, profile.delimiter);
                }
                if !profile.margins.1.is_zero() && index >= len - 1 - profile.margins.1 {
                    self.cut_off_margins.push_bottom(line, profile.delimiter);
                }
            }
        };
    }
}

impl From<DroppedFile> for FileToParse {
    fn from(value: DroppedFile) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            file: value,
            profile_name: None,
            profile: None,
            cut_off_margins: CutOffMargins::default(),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct CutOffMargins {
    top: Option<Vec<Vec<String>>>,
    bottom: Option<Vec<Vec<String>>>,
}

impl CutOffMargins {
    const MAX_STR_WIDTH: usize = 30;
    fn push_top(&mut self, str: &str, split: char) {
        let top = self.top.get_or_insert_with(Vec::default);
        top.push(
            str.split(split)
                .map(|str| {
                    if str.len() < Self::MAX_STR_WIDTH {
                        str.to_owned()
                    } else {
                        format!("{}...", &str[0..Self::MAX_STR_WIDTH].replace(" ", ""))
                    }
                })
                .collect(),
        );
    }
    fn push_bottom(&mut self, str: &str, split: char) {
        let bottom = self.bottom.get_or_insert_with(Vec::default);
        bottom.push(
            str.split(split)
                .map(|str| {
                    if str.len() < Self::MAX_STR_WIDTH {
                        str.to_owned()
                    } else {
                        format!("{}...", &str[0..Self::MAX_STR_WIDTH].replace(" ", ""))
                    }
                })
                .collect(),
        );
    }

    fn clear(&mut self) {
        self.top = None;
        self.bottom = None;
    }

    fn is_set(&self) -> bool {
        self.top.is_some() || self.bottom.is_some()
    }

    fn is_empty(&self) -> Option<bool> {
        match (&self.top, &self.bottom) {
            (Some(vec), None) | (None, Some(vec)) => Some(vec.is_empty()),
            (Some(a), Some(b)) => Some(a.is_empty() && b.is_empty()),
            _ => None,
        }
    }

    fn width(&self) -> Option<usize> {
        match (&self.top, &self.bottom) {
            (Some(vec), None) | (_, Some(vec)) => Some(vec.first().unwrap().len()),
            _ => None,
        }
    }
}

const DROPPED_FILES_EMPTY_TEXT: &str = "Drop in a file to see what happens";
