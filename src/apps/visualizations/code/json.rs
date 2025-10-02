mod merge;

use egui::{Color32, Ui};
use egui_json_tree::{
    DefaultExpand, JsonTree, JsonTreeStyle, JsonTreeVisuals, ToggleButtonsState,
};
use itertools::Itertools;
use serde_json::{Map, Value};

use crate::{
    apps::visualizations::code::json::merge::merge_all_maps,
    model::transactions::Transaction,
};

pub struct JsonExample {
    search_string: String,
    example_shape: Value,
}

impl JsonExample {
    pub fn from_trxn(trxns: &[Transaction]) -> Self {
        let example_shape = Self::create_example_shape(trxns);
        Self {
            search_string: Default::default(),
            example_shape,
        }
    }

    pub fn reset_trxns(&mut self, trxns: &[Transaction]) {
        self.example_shape = Self::create_example_shape(trxns);
    }

    pub fn show_json(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.search_string);
            if ui.button("Clear").clicked() {
                self.search_string.clear();
            }
        });

        let response = JsonTree::new("customised-tree", &self.example_shape)
            .default_expand(DefaultExpand::SearchResultsOrAll(
                &self.search_string,
            ))
            .style(
                JsonTreeStyle::new()
                    .abbreviate_root(true) // Show {...} when the root object is collapsed.
                    //.toggle_buttons_state(ToggleButtonsState::VisibleDisabled)
                    .visuals(JsonTreeVisuals {
                        bool_color: Color32::YELLOW,
                        ..Default::default()
                    }),
            )
            .show(ui);

        // By default, the tree will expand/collapse all arrays/objects
        // to respect the `default_expand` setting when it changes.
        // If required, you can manually trigger this reset, e.g. after a button press:
        if ui.button("Reset").clicked() {
            response.reset_expanded(ui);
        }
    }

    fn create_example_shape(trxns: &[Transaction]) -> Value {
        let values = trxns
            .iter()
            .map(|t| serde_json::to_value(t).unwrap())
            .filter_map(|v| match v {
                Value::Object(map) => Some(map),
                _ => None,
            });

        Value::Object(merge_all_maps(values))
    }
}
