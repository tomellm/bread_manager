use std::fs;

use data_communicator::buffered::{communicator::Communicator, query::QueryType};
use eframe::App;
use egui::ComboBox;
use egui_light_states::{default_promise_await::DefaultCreatePromiseAwait, UiStates};
use lazy_async_promise::{DirectCacheAccess, ImmediateValuePromise, ImmediateValueState};
use tokio::sync::mpsc;
use tracing::{warn, info};
use uuid::Uuid;

use crate::model::{
    linker::{Linker, PossibleLink},
    profiles::Profile,
    records::ExpenseRecord,
};

pub struct FileUpload {
    reciver: mpsc::Receiver<egui::DroppedFile>,
    update_callback_ctx: Option<egui::Context>,
    profiles: Communicator<Uuid, Profile>,
    records: Communicator<Uuid, ExpenseRecord>,
    possible_links: Communicator<Uuid, PossibleLink>,
    dropped_files: Vec<FileToParse>,
    parsed_records: ParsedRecords,
    ui: UiStates,
}

impl App for FileUpload {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_callback_ctx = Some(ctx.clone());
        self.recive_files();
        self.parsed_records.update();
        self.profiles.state_update();
        self.records.state_update();
        self.possible_links.state_update();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Files:");
            egui::Grid::new("uploaded files table").show(ui, |ui| {
                ui.label("name");
                ui.label("path");
                ui.label("profile");
                ui.label("remove");
                ui.end_row();
                self
                    .dropped_files
                    .retain_mut(|file_to_parse| {
                        ui.label(file_to_parse.file.name.clone());
                        if let Some(path) = file_to_parse.file.path.as_ref() {
                            ui.label(path.to_str().unwrap());
                        } else {
                            ui.label("..no path..");
                        }
                        ComboBox::new(format!("select_profile_{:?}", file_to_parse.file.path), "select profile")
                            .selected_text({
                                file_to_parse
                                    .profile
                                    .clone()
                                    .map_or(String::from("none selected"), |p| p.name)
                            })
                            .show_ui(ui, |ui| {
                                for profile in self.profiles.data_iter() {
                                    ui.selectable_value(
                                        &mut file_to_parse.profile,
                                        Some(profile.clone()),
                                        profile.name.clone(),
                                    );
                                }
                            });
                        if ui.button("remove").clicked() {
                            ui.end_row();
                            false 
                        } else {
                            ui.end_row();
                            true
                        }
                    });
            });
            ui.horizontal(|ui| {
                ui.heading("what to do now?");
                if ui.button("parse files").clicked() {
                    self.parse_files();
                }
                let promise = if ui.button("save parsed records").clicked() {
                    Some(self.save_parsed_data_action())
                } else {
                    None
                };
                self.ui
                    .default_promise_await("save_parsed_records".into())
                    .init_ui(|_, set_promise| {
                        if let Some(promise) = promise {
                            set_promise(promise);
                        }
                    })
                    .show(ui);
            });
            egui::Grid::new("expense records table").show(ui, |ui| {
                ui.label("amount");
                ui.label("time");
                ui.label("tags");
                ui.end_row();
                for record in &self.parsed_records.parsed_records {
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
    pub fn init(
        reciver: mpsc::Receiver<egui::DroppedFile>,
        profiles: Communicator<Uuid, Profile>,
        records: Communicator<Uuid, ExpenseRecord>,
        possible_links: Communicator<Uuid, PossibleLink>,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            let _ = profiles.query_future(QueryType::All).await;
            let _ = records.query_future(QueryType::All).await;
            // let _ = possible_links.query_future(QueryType::All).await;
            Self {
                reciver,
                dropped_files: vec![],
                update_callback_ctx: None,
                parsed_records: ParsedRecords::new(),
                profiles,
                records,
                possible_links,
                ui: UiStates::default(),
            }
        }
    }

    pub fn update_callback(&self) -> impl Fn() {
        let ctx = self.update_callback_ctx.clone().unwrap();
        move || ctx.request_repaint()
    }

    pub fn recive_files(&mut self) {
        while let Ok(file) = self.reciver.try_recv() {
            info!(msg = "Recived dropped file, adding to list", file = format!("{file:?}"));
            self.dropped_files.push(FileToParse::new(file));
        }
    }

    pub fn parse_files(&mut self) {
        self.parsed_records.parse_files(
            self.dropped_files
                .extract_if(|f| f.profile.is_some())
                .collect::<Vec<_>>(),
        );
    }

    pub fn save_parsed_data_action(&mut self) -> ImmediateValuePromise<()> {
        let records = self.parsed_records.drain_records();
        let links = Linker::find_links(records.iter().collect::<Vec<_>>(), self.records.data());

        let records_future = self.records.insert_many_future(records);
        let links_future = self.possible_links.insert_many_future(links);
        async move {
            let _ = records_future.await;
            let _ = links_future.await;
        }
        .into()
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
                panic!(
                    "Please select a profile for the file [{}] since no profile was selected.",
                    file.file.name
                );
            };

            let file = file.clone().path.unwrap();
            let str_file = fs::read_to_string(file).unwrap();
            let parsed_file = profile.parse_file(&str_file).unwrap();

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

    pub fn handle_parsed_records(&mut self, new_records: Vec<ExpenseRecord>) {
        self.parsed_records.extend(new_records);
    }

    pub fn update(&mut self) {
        let resulting_expenses = self
            .futures
            .extract_if(|working_future| {
                !matches!(working_future.poll_state(), ImmediateValueState::Updating)
            })
            .filter_map(|mut finished_future| {
                if let Some(result) = finished_future.take_result() {
                    match result {
                        Ok(expenses) => Some(expenses),
                        Err(err) => {
                            warn!(
                                "The parsing future did not succee. Failed with error [{:?}]",
                                *err
                            );
                            None
                        }
                    }
                } else {
                    None
                }
            })
            .flatten()
            .collect::<Vec<_>>();
        self.parsed_records.extend(resulting_expenses);
    }
    pub fn drain_records(&mut self) -> Vec<ExpenseRecord> {
        self.parsed_records.drain(..).collect()
    }
}
