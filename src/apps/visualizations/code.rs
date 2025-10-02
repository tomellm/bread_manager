pub(super) mod err;
pub(super) mod filter;
pub(super) mod json;

use egui::{Grid, Ui};
use egui_code_editor::{CodeEditor, Syntax};

use crate::{
    apps::visualizations::code::{
        err::CodeExecError, filter::CodeFilter, json::JsonExample,
    },
    model::transactions::Transaction,
};

/*
Example code

function (trxn)
    for _, t in ipairs(trxn['tags']) do
        if string.find(t['tag'], 'volksbank') then
            return true
        end
    end
    return false
end
*/

pub const CODE_FUNC: &str = r#"
function (trxn)
    return true;
end
"#;

pub struct CodeBlock {
    pub apply_code: bool,

    show_code: bool,
    code: String,

    show_example: bool,
    example: Option<JsonExample>,

    errors: Vec<CodeExecError>,
}

impl CodeBlock {
    pub fn show(&self) -> bool {
        self.show_code
    }

    pub fn toggle_show(&mut self) {
        self.show_code = !self.show_code;
    }

    pub fn create_filter(&self) -> CodeFilter {
        CodeFilter::from_code(&self.code)
    }

    pub fn reset_example_shape(&mut self, trxns: &[Transaction]) {
        if trxns.is_empty() {
            let _ = self.example.take();
            return;
        }

        match self.example.as_mut() {
            Some(example) => example.reset_trxns(trxns),
            None => {
                self.example = Some(JsonExample::from_trxn(trxns));
            }
        }
    }

    pub fn set_errors(&mut self, errors: Vec<CodeExecError>) {
        self.errors = errors;
    }

    pub fn update(&mut self, ui: &mut Ui) {
        if ui.button("clear").clicked() {
            self.code = CODE_FUNC.into();
        }
        if ui.button("show example").clicked() {
            self.show_example = true;
        }
        ui.separator();
        CodeEditor::default()
            .id_source("code_editor")
            .with_syntax(Syntax::lua())
            .with_numlines(true)
            .show(ui, &mut self.code);

        if self.show_example {
            egui::Window::new("Example Shape")
                .max_height(ui.available_height())
                .scroll([true, true])
                .open(&mut self.show_example)
                .show(ui.ctx(), |ui| {
                    match self.example.as_mut() {
                        Some(value) => value.show_json(ui),
                        None => {
                            ui.label("no transactions...");
                        }
                    };
                });
        }

        if !self.errors.is_empty() {
            let mut open = true;
            egui::Window::new("Errors")
                .scroll([true, true])
                .open(&mut open)
                .show(ui.ctx(), |ui| {
                    Grid::new("code errors grid").show(ui, |ui| {
                        self.errors.iter().for_each(|err| err.as_row(ui));
                    });
                });

            if !open {
                self.errors.clear();
            }
        }
    }
}

impl Default for CodeBlock {
    fn default() -> Self {
        Self {
            apply_code: false,
            show_code: false,
            code: CODE_FUNC.to_string(),
            show_example: false,
            example: None,
            errors: vec![],
        }
    }
}
