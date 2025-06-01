use crate::{
    components::expense_records::table::RecordsTable,
    db::query::{
        data_import_query::DataImportQuery, group_query::GroupsQuery,
        transaction_query::TransactionQuery,
    },
    model::{
        data_import::{row::ImportRow, DataImport},
        group::{Group, ModelGroup},
        profiles::{ParseResult, Profile},
        transactions::Transaction,
    },
    utils::PromiseUtilities,
};
use egui::{Grid, ScrollArea, Spinner, Ui};
use egui_light_states::UiStates;
use hermes::{
    carrier::execute::ImplExecuteCarrier,
    container::{data::ImplData, manual},
    factory::Factory,
};
use itertools::Itertools;
use lazy_async_promise::ImmediateValuePromise;
use std::{fs, mem, sync::Arc};
use tracing::info;

use super::{files_to_parse::FileToParse, ParsingFileState};

pub(super) struct ParsedRecords {
    transactions: manual::Container<Transaction>,
    imports: manual::Container<DataImport>,

    import_state: ImportParsingState,
    selected_overlay: usize,

    //linker: Linker,
    //columns_info: RecordsTable,
    //
    //ui: UiStates,
    //table: RecordsTable,
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
                //linker: Linker::init(factory).await,
                import_state: ImportParsingState::None,
                selected_overlay: 0,
                //columns_info: RecordsTable::default(),
                //ui: UiStates::default(),
                //table: RecordsTable::default(),
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
        //self.linker.state_update();

        if parsing_file.has_new_file() && self.import_state.ready_for_new() {
            let file_to_parse = parsing_file.start_parsing();
            self.find_overlaps(file_to_parse);
        }

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
                            self.start_parse();
                            parsing_file.finished_parsing();
                        }
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.add_enabled_ui(self.selected_overlay > 0, |ui| {
                            if ui.button("<").clicked() {
                                self.selected_overlay -= 1;
                            }
                        });
                        ui.label(format!(
                            "There are a total of {} overlaps!",
                            overlaps.overlaps.len()
                        ));
                        ui.add_enabled_ui(
                            self.selected_overlay < overlaps.overlaps.len() - 1,
                            |ui| {
                                if ui.button(">").clicked() {
                                    self.selected_overlay += 1;
                                }
                            },
                        );
                    });

                    let mut remove_this_overlap = false;
                    let mut there_was_overlap = false;
                    let mut check_overlapping = false;
                    let mut uncheck_all = false;

                    if !overlaps.is_overlap_cleared() {
                        Self::overlap_control_buttons(
                            &mut check_overlapping,
                            &mut uncheck_all,
                            &mut remove_this_overlap,
                            overlaps,
                            ui,
                        );
                    }

                    match overlaps.overlaps.get(self.selected_overlay) {
                        Some(overlap) => {
                            let max_len =
                                overlap.import.rows.len() + overlaps.rows.len();

                            ScrollArea::new(true).show(ui, |ui| {
                                Grid::new("overlap_grid").show(ui, |ui| {
                                    for index in 0..max_len {
                                        let overlap_record = (index
                                            >= overlap.first_match)
                                            .then(|| {
                                                overlap.import.rows.get(
                                                    index - overlap.first_match,
                                                )
                                            })
                                            .flatten();
                                        ui.label(
                                            overlap_record
                                                .map(|row| {
                                                    clamp_str(
                                                        row.row_content
                                                            .as_str(),
                                                    )
                                                })
                                                .unwrap_or_default(),
                                        );

                                        let mut import_record =
                                            overlaps.rows.get_mut(index);
                                        match import_record.as_mut() {
                                            Some(rec) => {
                                                if overlap_record.is_some()
                                                    && check_overlapping
                                                {
                                                    rec.0 = true;
                                                }
                                                if uncheck_all {
                                                    rec.0 = false;
                                                }
                                                ui.checkbox(&mut rec.0, "")
                                            }
                                            None => ui.label(""),
                                        };
                                        ui.label(
                                            import_record
                                                .as_ref()
                                                .map(|row| {
                                                    clamp_str(
                                                        row.1
                                                            .row_content
                                                            .as_str(),
                                                    )
                                                })
                                                .unwrap_or_default(),
                                        );
                                        ui.end_row();

                                        if overlap_record.is_none()
                                            && import_record.is_none()
                                        {
                                            return;
                                        } else if overlap_record.is_some()
                                            && import_record.is_some()
                                        {
                                            there_was_overlap = true;
                                        }
                                    }
                                });
                            });
                        }
                        None => {
                            self.selected_overlay = 0;
                            ui.label("... one moment, loading new overlap ...");
                        }
                    }
                    if !there_was_overlap || remove_this_overlap {
                        overlaps.remove_overlap(self.selected_overlay);
                    }
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
                    self.save_parse();
                }
            }
        });
    }

    fn find_overlaps(&mut self, file: FileToParse) {
        let FileToParse {
            file,
            profile: Some(profile),
            ..
        } = file
        else {
            unreachable!(
                "Please select a profile for the file [{}] since no profile was selected.",
                file.file.name
            );
        };

        let all_imports = Arc::clone(self.imports.data());

        let future = ImmediateValuePromise::new(async move {
            let file = file.path.unwrap();

            let file_str = fs::read_to_string(&file).unwrap();
            let import_rows = file_str
                .lines()
                .enumerate()
                .map(|(index, line)| ImportRow::init(line.to_string(), index))
                .collect_vec();

            let new_import = DataImport::init(profile.uuid, &file_str, file);

            let overlaps = all_imports
                .iter()
                .filter_map(|import| {
                    let mut first_match = None;
                    let sorted_counts = import
                        .rows
                        .iter()
                        .sorted_by_key(|row| row.row_index)
                        .counts_by(|row| {
                            for (index, new_row) in
                                new_import.rows.iter().enumerate()
                            {
                                if row.row_content.eq(&new_row.row_content) {
                                    if first_match.is_none() {
                                        first_match = Some(index);
                                    }
                                    return true;
                                }
                            }
                            false
                        });

                    let match_count = sorted_counts.get(&true).unwrap_or(&0);
                    match match_count {
                        0 => None,
                        _ => Some(ImportOverlap::new(
                            import.clone(),
                            *match_count,
                            first_match.unwrap(),
                        )),
                    }
                })
                .collect_vec();

            Ok(ImportResultWithOverlap::new(
                new_import,
                import_rows,
                profile,
                overlaps,
            ))
        });
        self.import_state.set_overlaps(future);
    }

    fn start_parse(&mut self) {
        let ImportParsingState::OverlapsFound(overlaps) =
            mem::replace(&mut self.import_state, ImportParsingState::None)
        else {
            unreachable!()
        };
        self.import_state.set_parsing(
            async move {
                let ImportResultWithOverlap {
                    mut import,
                    profile,
                    rows,
                    ..
                } = overlaps;

                // ToDo: ignore the columns that are clicked to ignore
                import.rows = rows.into_iter().map(|t| t.1).collect();
                let ParseResult {
                    rows,
                    groups,
                    import,
                } = profile.parse_file(import).unwrap();
                (rows, import, groups)
            }
            .into(),
        );
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
                .execute(diq_1)
                .execute(diq_2)
                .execute(diq_3);
        });
    }

    fn overlap_control_buttons(
        check_overlapping: &mut bool,
        uncheck_all: &mut bool,
        remove_this_overlap: &mut bool,
        overlaps: &mut ImportResultWithOverlap,
        ui: &mut Ui,
    ) {
        ui.horizontal(|ui| {
            *check_overlapping = ui.button("check all overlapping").clicked();
            *uncheck_all = ui.button("uncheck all").clicked();
            *remove_this_overlap = ui.button("remove this overlap").clicked();
            if ui.button("remove all checked").clicked() {
                overlaps.rows =
                    overlaps.rows.drain(..).filter(|row| !row.0).collect_vec()
            }
        });
    }
}

pub enum ImportParsingState {
    None,
    FindingOverlaps(ImmediateValuePromise<ImportResultWithOverlap>),
    OverlapsFound(ImportResultWithOverlap),
    Parsing(ImmediateValuePromise<(Vec<Transaction>, DataImport, Vec<Group>)>),
    Finished(Vec<Transaction>, DataImport, Vec<Group>),
}

impl ImportParsingState {
    fn ready_for_new(&self) -> bool {
        matches!(self, ImportParsingState::None)
    }
    fn set_overlaps(
        &mut self,
        future: ImmediateValuePromise<ImportResultWithOverlap>,
    ) {
        assert!(matches!(self, Self::None));
        let _ = mem::replace(self, ImportParsingState::FindingOverlaps(future));
    }

    fn set_parsing(
        &mut self,
        future: ImmediateValuePromise<(
            Vec<Transaction>,
            DataImport,
            Vec<Group>,
        )>,
    ) {
        let _ = mem::replace(self, ImportParsingState::Parsing(future));
    }
    fn try_resolve(&mut self) {
        if let Self::FindingOverlaps(finding) = self {
            finding
                .poll_and_check_finished()
                .then(|| finding.take_expect())
        } else {
            None
        }
        .map(|value| mem::replace(self, Self::OverlapsFound(value)));

        if let Self::Parsing(parsing) = self {
            parsing
                .poll_and_check_finished()
                .then(|| parsing.take_expect())
        } else {
            None
        }
        .map(|value| {
            mem::replace(self, Self::Finished(value.0, value.1, value.2))
        });
    }
    fn clear(&mut self) {
        let _ = mem::replace(self, ImportParsingState::None);
    }
}

pub struct ImportResultWithOverlap {
    import: DataImport,
    rows: Vec<(bool, ImportRow)>,
    profile: Profile,
    overlaps: Vec<ImportOverlap>,
}

impl ImportResultWithOverlap {
    pub fn new(
        import: DataImport,
        rows: Vec<ImportRow>,
        profile: Profile,
        overlaps: Vec<ImportOverlap>,
    ) -> Self {
        Self {
            import,
            rows: rows.into_iter().map(|rec| (false, rec)).collect(),
            profile,
            overlaps,
        }
    }

    pub fn remove_overlap(&mut self, index: usize) {
        info!("this ran");
        self.overlaps.remove(index);
    }

    pub fn is_overlap_cleared(&self) -> bool {
        self.overlaps.is_empty()
    }
}

pub struct ImportOverlap {
    import: DataImport,
    match_count: usize,
    first_match: usize,
}

impl ImportOverlap {
    pub fn new(
        import: DataImport,
        match_count: usize,
        first_match: usize,
    ) -> Self {
        Self {
            import,
            match_count,
            first_match,
        }
    }
}

fn clamp_str(str: &str) -> &str {
    match str.len() <= 30 {
        true => str,
        false => &str[0..30],
    }
}

const PARSED_RECORDS_EMPTY_TEXT: &str = r#"
Drop in some files, select a profile and then click on 'Parse Files' to preview the parsed records
before clicking on 'Save parsed Data' to save them.
"#;
