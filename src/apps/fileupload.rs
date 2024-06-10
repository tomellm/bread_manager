use std::fs;

use egui::ComboBox;
use lazy_async_promise::{ImmediateValuePromise, ImmediateValueState};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    model::{profiles::Profile, records::ExpenseRecord},
    utils::{communicator::Communicator, misc},
};

pub struct FileUpload {
    reciver: mpsc::Receiver<egui::DroppedFile>,
    update_callback_ctx: Option<egui::Context>,
    profiles_communicator: Communicator<Uuid, Profile>,
    records_communicator: Communicator<Uuid, ExpenseRecord>,
    dropped_files: Vec<FileToParse>,
    parsed_records: ParsedRecords,
}

impl eframe::App for FileUpload {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_callback_ctx = Some(ctx.clone());
        self.recive_files();
        self.parsed_records.update();
        self.profiles_communicator.update();
        self.records_communicator.update();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Files:");
            egui::Grid::new("uploaded files table").show(ui, |ui| {
                ui.label("number");
                ui.label("name");
                ui.label("path");
                ui.label("profile");
                ui.label("remove");
                ui.end_row();
                let mut to_delete = vec![];
                for (index, file_to_parse) in self.dropped_files.iter_mut().enumerate() {
                    ui.label(format!("{index}"));
                    ui.label(file_to_parse.file.name.clone());
                    if let Some(path) = file_to_parse.file.path.as_ref() {
                        ui.label(path.to_str().unwrap());
                    } else {
                        ui.label("..no path..");
                    }
                    ComboBox::new(format!("select_profile_{}", index), "select profile")
                        .selected_text({
                            file_to_parse
                                .profile
                                .clone()
                                .map(|p| p.name)
                                .unwrap_or(String::from("none selected"))
                        })
                        .show_ui(ui, |ui| {
                            for (_, profile) in self.profiles_communicator.view().iter() {
                                ui.selectable_value(
                                    &mut file_to_parse.profile,
                                    Some(profile.clone()),
                                    profile.name.clone(),
                                );
                            }
                        });
                    if ui.button("remove").clicked() {
                        to_delete.push(index);
                    }
                    ui.end_row();
                }
                misc::clear_vec(to_delete, &mut self.dropped_files);
            });
            ui.horizontal(|ui| {
                ui.heading("what to do now?");
                if ui.button("parse files").clicked() {
                    self.parse_files();
                }
                if ui.button("save parsed records").clicked() {
                    self.save_parsed_data();
                }
            });
            egui::Grid::new("expense records table").show(ui, |ui| {
                ui.label("amount");
                ui.label("time");
                ui.label("tags");
                ui.end_row();
                for record in self.parsed_records.parsed_records.iter() {
                    ui.label(format!("{}", record.amount()));
                    ui.label(format!("{}", record.datetime()));
                    ui.label(format!("{:?}", record.tags()));
                    ui.end_row();
                }
            });
        });
    }
}

impl FileUpload {
    pub fn new(
        reciver: mpsc::Receiver<egui::DroppedFile>,
        profiles_communicator: Communicator<Uuid, Profile>,
        records_communicator: Communicator<Uuid, ExpenseRecord>,
    ) -> Self {
        Self {
            reciver,
            dropped_files: vec![],
            update_callback_ctx: None,
            parsed_records: ParsedRecords::new(),
            profiles_communicator,
            records_communicator,
        }
    }

    pub fn update_callback(&self) -> impl Fn() {
        let ctx = self.update_callback_ctx.clone().unwrap();
        move || ctx.request_repaint()
    }

    pub fn recive_files(&mut self) {
        while let Ok(file) = self.reciver.try_recv() {
            self.dropped_files.push(FileToParse::new(file));
            self.update_callback()();
        }
    }

    pub fn parse_files(&mut self) {
        self.parsed_records.parse_files(
            self.dropped_files
                .extract_if(|f| f.profile.is_some())
                .collect::<Vec<_>>(),
        );
    }

    pub fn save_parsed_data(&mut self) {
        let records = self.parsed_records.drain_records();
        self.records_communicator.set_many(records);
    }

    pub fn show_file_viewer() -> bool {
        true
    }
}

#[derive(Clone, Debug)]
struct FileToParse {
    pub file: egui::DroppedFile,
    pub profile: Option<Profile>,
}

impl FileToParse {
    pub fn new(file: egui::DroppedFile) -> Self {
        Self {
            file,
            profile: None,
        }
    }
}

struct ParsedRecords {
    parsed_records: Vec<ExpenseRecord>,
    futures: Vec<ImmediateValuePromise<Vec<ExpenseRecord>>>,
}

impl ParsedRecords {
    pub fn new() -> Self {
        Self {
            parsed_records: vec![],
            futures: vec![],
        }
    }
    fn create_future(file: FileToParse) -> ImmediateValuePromise<Vec<ExpenseRecord>> {
        ImmediateValuePromise::new(async move {
            let FileToParse {
                file,
                profile: Some(profile),
            } = file
            else {
                panic!("hello???");
            };

            let file = file.clone().path.unwrap();
            let str_file = fs::read_to_string(file).unwrap();
            let parsed_file = profile.parse_file(str_file).unwrap();

            Ok(parsed_file)
        })
    }

    pub fn parse_file(&mut self, file: FileToParse) {
        self.futures.push(Self::create_future(file));
    }
    pub fn parse_files(&mut self, files: Vec<FileToParse>) {
        let futures = files
            .into_iter()
            .map(Self::create_future)
            .collect::<Vec<_>>();
        self.futures.extend(futures);
    }

    pub fn update(&mut self) {
        self.futures.retain_mut(|future| match future.poll_state() {
            ImmediateValueState::Empty => false,
            ImmediateValueState::Error(_) => panic!("Error completing future!"),
            ImmediateValueState::Updating => true,
            ImmediateValueState::Success(vec) => {
                self.parsed_records.extend(vec.clone());
                false
            }
        });
    }
    pub fn drain_records(&mut self) -> Vec<ExpenseRecord> {
        self.parsed_records.drain(..).collect()
    }
}
