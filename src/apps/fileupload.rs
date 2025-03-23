mod files_to_parse;
mod parsed_records;

use std::mem;

use eframe::App;
use egui_light_states::UiStates;
use files_to_parse::{FileToParse, FilesToParse};
use hermes::factory::Factory;
use parsed_records::ParsedRecords;
use tokio::sync::mpsc;

pub struct FileUpload {
    parsed_records: ParsedRecords,
    files_to_parse: FilesToParse,

    parsing_file: ParsingFileState,

    ui: UiStates,
}

impl App for FileUpload {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Files:");
            self.files_to_parse
                .files_to_parse_list(&mut self.parsing_file, ui);
            self.parsed_records.ui_update(&mut self.parsing_file, ui);
        });
    }
}

impl FileUpload {
    pub fn init(
        reciver: mpsc::Receiver<egui::DroppedFile>,
        factory: Factory,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            Self {
                files_to_parse: FilesToParse::init(reciver, &factory).await,
                parsed_records: ParsedRecords::init(factory).await,
                parsing_file: ParsingFileState::Empty,
                ui: UiStates::default(),
            }
        }
    }

    pub fn parse_files(&mut self) {}
}

enum ParsingFileState {
    Empty,
    Parsing,
    NewFile(Box<FileToParse>),
}

impl ParsingFileState {
    pub fn ready_for_parse(&self) -> bool {
        matches!(self, ParsingFileState::Empty)
    }

    pub fn has_new_file(&self) -> bool {
        matches!(self, ParsingFileState::NewFile(_))
    }

    pub fn insert(&mut self, file: FileToParse) {
        assert!(matches!(self, ParsingFileState::Empty));
        let _ = mem::replace(self, ParsingFileState::NewFile(Box::new(file)));
    }

    pub fn start_parsing(&mut self) -> FileToParse {
        let ParsingFileState::NewFile(boxed) =
            mem::replace(self, ParsingFileState::Parsing)
        else {
            unreachable!()
        };
        *boxed
    }

    pub fn finished_parsing(&mut self) {
        let _ = mem::replace(self, ParsingFileState::Empty);
    }
}

const TOOL_TIP_PARSE_FILES: &str = r#"Parse all the files for which you have selected a profile. 
Will not save them yet, for that click 'Save parsed Data'."#;
