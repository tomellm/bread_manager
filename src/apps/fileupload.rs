mod files_to_parse;
mod parsed_records;

use eframe::App;
use egui_light_states::UiStates;
use files_to_parse::FilesToParse;
use hermes::factory::Factory;
use parsed_records::ParsedRecords;
use tokio::sync::mpsc;

pub struct FileUpload {
    parsed_records: ParsedRecords,
    files_to_parse: FilesToParse,
    ui: UiStates,
}

impl App for FileUpload {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Files:");
            self.files_to_parse.files_to_parse_list(ui);
            if !self.files_to_parse.is_empty() {
                ui.vertical_centered(|ui| {
                    if ui
                        .button("Parse files")
                        .on_hover_text(TOOL_TIP_PARSE_FILES)
                        .clicked()
                    {
                        self.parse_files();
                    }
                    ui.label("");
                    ui.label("v");
                });
            }
            self.parsed_records.ui_update(ui);
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
