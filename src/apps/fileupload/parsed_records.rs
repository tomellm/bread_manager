mod find_overlaps;
mod import_parsing_state;
mod results_with_overlaps;

use crate::{
    apps::fileupload::parsed_records::{
        find_overlaps::find_overlaps, results_with_overlaps::ImportOverlap,
    },
    components::expense_records::table::TransactsTable,
    db::query::{
        data_import_query::DataImportQuery, group_query::GroupsQuery,
        transaction_query::TransactionQuery,
    },
    model::{
        data_import::{row::ImportRow, DataImport},
        group::ModelGroup,
        transactions::Transaction,
    },
};
use egui::{Grid, ScrollArea, Spinner, Ui};
use hermes::{
    carrier::execute::ImplExecuteCarrier, container::manual, factory::Factory,
};
use import_parsing_state::ImportParsingState;
use results_with_overlaps::{
    overlap_control_buttons, ImportResultWithOverlap, RowSelectionStatus,
};
use std::mem;

use super::ParsingFileState;

pub(super) struct ParsedRecords {
    transactions: manual::Container<Transaction>,
    imports: manual::Container<DataImport>,

    import_state: ImportParsingState,
    selected_overlay: usize,
    transacts_table: TransactsTable,
}

impl ParsedRecords {
    pub(super) fn init(
        factory: Factory,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            let transactions = factory.builder().file(file!()).manual();
            let mut imports = factory.builder().file(file!()).manual();
            imports.stored_query(DataImportQuery::all);

            Self {
                transactions,
                imports,
                import_state: ImportParsingState::None,
                selected_overlay: 0,
                transacts_table: TransactsTable::default(),
            }
        }
    }

    pub fn ui_update(
        &mut self,
        parsing_file: &mut ParsingFileState,
        ui: &mut Ui,
    ) {
        self.import_state.try_resolve();

        self.transactions.state_update(false);
        self.imports.state_update(true);

        if parsing_file.has_new_file() && self.import_state.ready_for_new() {
            let file_to_parse = parsing_file.start_parsing();
            find_overlaps(self, file_to_parse);
        }
        let mut clear_parse = false;
        let mut save_parse = false;
        ui.vertical_centered(|ui| match &mut self.import_state {
            ImportParsingState::None => {
                ui.label(PARSED_RECORDS_EMPTY_TEXT);
            }
            ImportParsingState::Parsing(_)
            | ImportParsingState::FindingOverlaps(_) => {
                ui.add(Spinner::new());
            }
            ImportParsingState::OverlapsFound(overlaps) => {
                if overlaps.overlaps.is_empty() {
                    ui.vertical(|ui| {
                        ui.label("There are no overlaps to resolve");
                        if ui.button("parse file").clicked() {
                            self.import_state.start_parse();
                            parsing_file.finished_parsing();
                        }
                    });
                } else {
                    clear_parse =
                        show_overlaps(overlaps, &mut self.selected_overlay, ui);
                }
            }
            ImportParsingState::Finished(transactions, data_import, groups) => {
                ui.heading("Final Stats");
                ui.label(format!(
                    "Num of Transactions: {}",
                    transactions.len()
                ));
                ui.label(format!(
                    "Num of Rows in DataImport: {}",
                    data_import.rows.len()
                ));
                ui.label(format!("Number of Groups: {}", groups.len()));
                if ui.button("save").clicked() {
                    save_parse = true;
                }
                self.transacts_table.show(transactions, ui);
            }
        });
        if save_parse {
            self.save_parse();
        }
    }

    fn save_parse(&mut self) {
        let ImportParsingState::Finished(transacts, import, groups) =
            mem::replace(&mut self.import_state, ImportParsingState::None)
        else {
            unreachable!();
        };

        let tr_q =
            manual::Container::<Transaction>::insert_many_queries(transacts);
        let (diq_1, diq_2, diq_3) =
            manual::Container::<DataImport>::insert_queries(vec![import]);

        self.transactions.execute_many(|transac| {
            transac.execute(
                manual::Container::<ModelGroup>::insert_many_query(groups),
            );
            tr_q.add_all_to_transaction(transac)
                .execute_many(diq_1)
                .execute_many(diq_2)
                .execute_many(diq_3);
        });
    }
}

fn show_overlaps(
    overlaps: &mut ImportResultWithOverlap,
    selected_overlay: &mut usize,
    ui: &mut Ui,
) -> bool {
    selected_overlay_controls(selected_overlay, overlaps.overlaps.len(), ui);

    let mut clear_parse = false;
    let mut remove_this_overlap = false;
    let mut there_was_overlap = false;
    let mut check_overlapping = false;
    let mut uncheck_all = false;

    if !overlaps.is_overlap_cleared() {
        clear_parse = overlap_control_buttons(
            &mut check_overlapping,
            &mut uncheck_all,
            &mut remove_this_overlap,
            ui,
        );
    }

    match overlaps.overlaps.get_mut(*selected_overlay) {
        Some(overlap) => {
            let max_len = overlap.import.rows.len() + overlaps.rows.len();

            ScrollArea::both().show(ui, |ui| {
                Grid::new("overlap_grid").show(ui, |ui| {
                    header_row(ui);

                    for index in 0..max_len {
                        let existing_is_some = existing_row(index, overlap, ui);

                        let mut import_record = overlaps.rows.get_mut(index);
                        checkbox(
                            existing_is_some,
                            &mut import_record,
                            &mut check_overlapping,
                            &mut uncheck_all,
                            ui,
                        );
                        new_row(&mut import_record, ui);

                        ui.end_row();

                        if !existing_is_some && import_record.is_none() {
                            return;
                        } else if existing_is_some && import_record.is_some() {
                            there_was_overlap = true;
                        }
                    }
                });
            });
        }
        None => {
            *selected_overlay = 0;
            ui.label("... one moment, loading new overlap ...");
        }
    }
    if !there_was_overlap || remove_this_overlap {
        overlaps.remove_overlap(*selected_overlay);
    }
    clear_parse
}

fn header_row(ui: &mut Ui) {
    ui.label("");
    ui.label("Existing Records: ");
    ui.label("|");
    ui.label("");
    ui.label("New Records:");
    ui.end_row();
}

fn selected_overlay_controls(
    selected_overlay: &mut usize,
    overlays_len: usize,
    ui: &mut Ui,
) {
    ui.horizontal(|ui| {
        ui.add_enabled_ui(*selected_overlay > 0, |ui| {
            if ui.button("<").clicked() {
                *selected_overlay -= 1;
            }
        });
        ui.label(format!("There are a total of {overlays_len} overlaps!"));
        ui.add_enabled_ui(*selected_overlay < overlays_len - 1, |ui| {
            if ui.button(">").clicked() {
                *selected_overlay += 1;
            }
        });
    });
}

fn existing_row(index: usize, overlap: &ImportOverlap, ui: &mut Ui) -> bool {
    let overlap_import_row = (index >= overlap.first_match)
        .then(|| overlap.import.rows.get(index - overlap.first_match))
        .flatten();
    ui.label(
        overlap_import_row
            .map(|r| r.row_index.to_string())
            .unwrap_or_default(),
    );
    ui.label(
        overlap_import_row
            .map(|row| clamp_str(row.row_content.as_str()))
            .unwrap_or_default(),
    );
    overlap_import_row.is_some()
}

fn checkbox(
    existing_row_is_some: bool,
    import_record: &mut Option<&mut (RowSelectionStatus, ImportRow)>,
    check_overlapping: &mut bool,
    uncheck_all: &mut bool,
    ui: &mut Ui,
) {
    match import_record {
        Some(rec) => {
            if existing_row_is_some && *check_overlapping {
                rec.0.set(true);
            }
            if *uncheck_all {
                rec.0.set(false);
            }
            ui.add_enabled_ui(rec.0.include.is_some(), |ui| {
                ui.checkbox(rec.0.as_mut().unwrap_or(&mut false), "");
            })
            .response
            .on_hover_text(OVERLAP_CECKBOX_TEXT)
            .on_disabled_hover_text(OVERLAP_CHECKBOX_DISABLED_TEXT);
        }
        None => {
            ui.label("");
        }
    };
}

fn new_row(
    import_record: &mut Option<&mut (RowSelectionStatus, ImportRow)>,
    ui: &mut Ui,
) {
    ui.label(
        import_record
            .as_ref()
            .map(|r| r.1.row_index.to_string())
            .unwrap_or_default(),
    );
    ui.label(
        import_record
            .as_ref()
            .map(|row| clamp_str(row.1.row_content.as_str()))
            .unwrap_or_default(),
    );
}

fn clamp_str(str: &str) -> &str {
    const WIDTH: usize = 100;

    match str.len() <= WIDTH {
        true => str,
        false => &str[0..WIDTH],
    }
}

const PARSED_RECORDS_EMPTY_TEXT: &str = r#"
Drop in some files, select a profile and then click on 'Parse Files' to preview the parsed records
before clicking on 'Save parsed Data' to save them.
"#;

const OVERLAP_CECKBOX_TEXT: &str = r#"
With this checkbox you can decide if this row will be parsed or not. Check it
to include it in the parse, Uncheck it to exclude it.
"#;

const OVERLAP_CHECKBOX_DISABLED_TEXT: &str = r#"
This Checkbox is disabled as is it is part of the Margins mentioned in the 
Profile settings. These rows are only visible for completions sake
"#;
