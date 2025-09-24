use egui::{Grid, Ui};
use egui_code_editor::{CodeEditor, Syntax};
use egui_extras::syntax_highlighting;
use mlua::{Function, Lua, LuaSerdeExt};
use tracing::info;

use crate::model::transactions::Transaction;

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
    example_shape: String,

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
            return;
        }

        self.example_shape =
            serde_json::to_string_pretty(trxns.first().unwrap()).unwrap();
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
                .open(&mut self.show_example)
                .show(ui.ctx(), |ui| {
                    let language = "json";
                    let theme = syntax_highlighting::CodeTheme::from_memory(
                        ui.ctx(),
                        ui.style(),
                    );
                    syntax_highlighting::code_view_ui(
                        ui,
                        &theme,
                        &self.example_shape,
                        language,
                    );
                });
        }

        if !self.errors.is_empty() {
            let mut open = true;
            egui::Window::new("Errors")
                .scroll([true, true])
                .open(&mut open)
                .show(ui.ctx(), |ui| {
                    Grid::new("code errors grid").show(ui, |ui| {
                        self.errors.iter().for_each(|err| {
                            ui.label(err.err.to_string());
                            ui.label(err.trxn.uuid.to_string());
                            ui.end_row();
                        });
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
            example_shape: String::default(),
            errors: vec![],
        }
    }
}

pub struct CodeFilter {
    lua: Lua,
    func: Function,
}

impl CodeFilter {
    pub fn from_code(code: &str) -> Self {
        let lua = Lua::new();
        let func: Function = lua
            .load(code)
            .eval()
            .map_err(|err| {
                tracing::error!(
                    "error while loading the filter function: {err:?}"
                );
            })
            .unwrap();
        Self { lua, func }
    }

    pub fn call(&self, trxn: &Transaction) -> Result<bool, mlua::Error> {
        let val_trxn = self.lua.to_value(trxn).unwrap();
        self.func.call::<bool>(val_trxn)
    }
}

#[derive(Debug)]
pub struct CodeExecError {
    trxn: Transaction,
    err: String,
}

impl CodeExecError {
    pub fn new(trxn: &Transaction, err: mlua::Error) -> Self {
        Self {
            trxn: trxn.clone(),
            err: err.to_string(),
        }
    }
}
