mod files_to_parse;
mod parsed_records;

use data_communicator::buffered::communicator::Communicator;
use eframe::App;
use egui_light_states::UiStates;
use files_to_parse::FilesToParse;
use parsed_records::ParsedRecords;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::model::{
    linker::{Link, PossibleLink},
    profiles::Profile,
    records::ExpenseRecord,
};

pub struct FileUpload {
    parsed_records: ParsedRecords,
    files_to_parse: FilesToParse,
    ui: UiStates,
}

impl App for FileUpload {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Files:");
                ui.add_space(20.);
                ui.horizontal(|ui| {
                    if ui.button("Parse files")
                        .on_hover_text(TOOL_TIP_PARSE_FILES)
                        .clicked() {
                        self.parse_files();
                    }
                });
            });
            self.files_to_parse.files_to_parse_list(ui);

            ui.add_space(20.);
            ui.separator();
            ui.add_space(20.);
            self.parsed_records.ui_update(ui);
        });
    }
}

impl FileUpload {
    pub fn init(
        reciver: mpsc::Receiver<egui::DroppedFile>,
        records: [Communicator<Uuid, ExpenseRecord>; 2],
        profiles: Communicator<Uuid, Profile>,
        possible_links: Communicator<Uuid, PossibleLink>,
        links: Communicator<Uuid, Link>,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            Self {
                files_to_parse: FilesToParse::init(reciver, profiles).await,
                parsed_records: ParsedRecords::init(records, possible_links, links).await,
                ui: UiStates::default(),
            }
        }
    }

    pub fn parse_files(&mut self) {
        self.parsed_records
            .parse_files(self.files_to_parse.extract_ready_files());
    }
}

const TOOL_TIP_PARSE_FILES: &str = r#"Parse all the files for which you have selected a profile. 
Will not save them yet, for that click 'Save parsed Data'."#;
