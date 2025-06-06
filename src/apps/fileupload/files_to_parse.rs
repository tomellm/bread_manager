mod margin_cutoff;

use std::fs;

use egui::{ComboBox, DroppedFile, Grid, Ui};
use hermes::{
    container::{data::ImplData, manual},
    factory::Factory,
};
use margin_cutoff::{margin_cutoff, CutOffMargins};
use num_traits::Zero;
use tokio::sync::mpsc;
use tracing::info;
use uuid::Uuid;

use crate::{db::query::profile_query::ProfileQuery, model::profiles::Profile};

use super::ParsingFileState;

pub(super) struct FilesToParse {
    reciver: mpsc::Receiver<DroppedFile>,
    profiles: manual::Container<Profile>,
    files: Vec<FileToParse>,
}

impl FilesToParse {
    pub(super) fn init(
        reciver: mpsc::Receiver<DroppedFile>,
        factory: &Factory,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        let mut profiles = factory.builder().file(file!()).manual();

        async move {
            profiles.stored_query(ProfileQuery::all_active);

            Self {
                reciver,
                profiles,
                files: vec![],
            }
        }
    }

    pub(super) fn files_to_parse_list(
        &mut self,
        parsing_file: &mut ParsingFileState,
        ui: &mut Ui,
    ) {
        self.profiles.state_update(true);
        self.recive_files();

        if !self.files.is_empty() {
            egui::Grid::new("uploaded files table").show(ui, |ui| {
                ui.label("name");
                ui.label("path");
                ui.label("profile");
                ui.label("margin\ncutoff");
                ui.label("remove");
                ui.end_row();
                self.files.retain_mut(|file_to_parse| {
                    file_to_parse.update_internal_values();

                    ui.label(file_to_parse.file.name.clone());
                    Self::file_path(file_to_parse, ui);
                    Self::profile_select(
                        file_to_parse,
                        self.profiles.data(),
                        ui,
                    );
                    margin_cutoff(file_to_parse, ui);
                    Self::parse_and_remove_button(
                        file_to_parse,
                        parsing_file,
                        ui,
                    )
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

    fn profile_select(
        file: &mut FileToParse,
        profiles: &[Profile],
        ui: &mut Ui,
    ) {
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

    fn parse_and_remove_button(
        file_to_parse: &mut FileToParse,
        parsing_file: &mut ParsingFileState,
        ui: &mut Ui,
    ) -> bool {
        let mut to_remove = true;

        if ui.button("remove").clicked() {
            to_remove = false;
        }

        ui.add_enabled_ui(
            parsing_file.ready_for_parse() && file_to_parse.profile.is_some(),
            |ui| {
                if ui.button("parse file").clicked() {
                    parsing_file.insert(file_to_parse.clone());
                    to_remove = false;
                }
            },
        );

        ui.end_row();
        to_remove
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

            str.lines().enumerate().for_each(|(index, line)| {
                let is_top = profile.is_top_margin(index);
                #[allow(clippy::nonminimal_bool)]
                if is_top || (!is_top && profile.is_top_margin(index - 1)) {
                    self.cut_off_margins.push_top(line, profile.delimiter);
                }
                let is_btm = profile.is_bottom_margin(index, len);
                #[allow(clippy::nonminimal_bool)]
                if is_btm || (!is_btm && profile.is_bottom_margin(index + 1, len)) {
                    self.cut_off_margins.push_bottom(line, profile.delimiter);
                }
            });
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

const DROPPED_FILES_EMPTY_TEXT: &str = "Drop in a file to see what happens";
