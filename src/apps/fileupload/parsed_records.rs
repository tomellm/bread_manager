use crate::{
    components::expense_records::table::RecordsTable,
    db::{
        self, data_import::DbDataImport, possible_links::DbPossibleLink,
        records::DbRecord,
    },
    model::{
        data_import::DataImport, linker::Linker, profiles::ParseResult,
        records::ExpenseRecord,
    },
    utils::PromiseUtilities,
};
use egui::{Grid, ScrollArea, Spinner, Ui};
use egui_light_states::UiStates;
use hermes::{
    carrier::{execute::ImplExecuteCarrier, query::ImplQueryCarrier},
    container::projecting::ProjectingContainer,
    factory::Factory,
    ToActiveModel,
};
use itertools::Itertools;
use lazy_async_promise::{DirectCacheAccess, ImmediateValuePromise};
use sea_orm::{ColumnTrait, EntityOrSelect, EntityTrait, QueryFilter};
use std::{collections::HashMap, fs, mem};
use tracing::info;
use uuid::Uuid;

use super::{files_to_parse::FileToParse, ParsingFileState};

pub(super) struct ParsedRecords {
    records: ProjectingContainer<ExpenseRecord, DbRecord>,

    import_state: ImportParsingState,
    selected_overlay: usize,

    linker: Linker,
    columns_info: RecordsTable,

    ui: UiStates,
    table: RecordsTable,
}

impl ParsedRecords {
    pub(super) fn init(
        factory: Factory,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            let mut records =
                factory.builder().name("parse_records_records").projector();

            records.stored_query(DbRecord::find().select());

            Self {
                records,
                linker: Linker::init(factory).await,
                import_state: ImportParsingState::None,
                selected_overlay: 0,
                columns_info: RecordsTable::default(),
                ui: UiStates::default(),
                table: RecordsTable::default(),
            }
        }
    }

    pub fn ui_update(
        &mut self,
        parsing_file: &mut ParsingFileState,
        ui: &mut Ui,
    ) {
        self.import_state.try_resolve();

        self.records.state_update(true);
        self.linker.state_update();

        if parsing_file.has_new_file() && self.import_state.ready_for_new() {
            let file_to_parse = parsing_file.start_parsing();
            self.parse_file(file_to_parse);
        }

        ui.vertical_centered(|ui| match &mut self.import_state {
            ImportParsingState::Parsing(_) => {
                ui.add(Spinner::new());
            }
            ImportParsingState::Finished(import_result) => {
                if import_result.overlaps.is_empty() {
                    ui.vertical(|ui| {
                        ui.label("There are no overlaps to resolve");
                        if ui.button("save parsed values").clicked() {
                            self.save_parse();
                            self.import_state.clear();
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
                            import_result.overlaps.len()
                        ));
                        ui.add_enabled_ui(
                            self.selected_overlay
                                < import_result.overlaps.len() - 1,
                            |ui| {
                                if ui.button(">").clicked() {
                                    self.selected_overlay += 1;
                                }
                            },
                        );
                    });

                    let mut remove_this_overlap = false;
                    let mut there_was_overlap = false;
                    match import_result.overlaps.get(self.selected_overlay) {
                        Some(overlap) => {
                            let mut check_overlapping = false;
                            let mut uncheck_all = false;
                            ui.horizontal(|ui| {
                                check_overlapping = ui
                                    .button("check all overlapping")
                                    .clicked();
                                uncheck_all =
                                    ui.button("uncheck all").clicked();
                                remove_this_overlap =
                                    ui.button("remove this overlap").clicked();
                                if ui.button("remove all checked").clicked() {
                                    import_result.rows = import_result
                                        .rows
                                        .drain(..)
                                        .filter(|row| !row.0)
                                        .collect_vec()
                                }
                            });

                            let max_len = overlap.records.len()
                                + import_result.rows.len();

                            ScrollArea::new(true).show(ui, |ui| {
                                Grid::new("overlap_grid").show(ui, |ui| {
                                    for index in 0..max_len {
                                        let overlap_record = (index
                                            >= overlap.first_match)
                                            .then(|| {
                                                overlap.records.get(
                                                    index - overlap.first_match,
                                                )
                                            })
                                            .flatten();
                                        Self::display_record_row(
                                            overlap_record,
                                            ui,
                                        );

                                        let mut import_record =
                                            import_result.rows.get_mut(index);
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
                                        Self::display_record_row(
                                            import_record
                                                .as_ref()
                                                .map(|rec| &rec.1),
                                            ui,
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
                        import_result.remove_overlap(self.selected_overlay);
                    }
                }
            }
            _ => (),
        });
    }

    fn parse_file(&mut self, file: FileToParse) {
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

        let query = self.records.direct_proj_query(
            DbRecord::find()
                .filter(
                    db::records::Column::Origin.eq(profile.origin_name.clone()),
                )
                .select(),
        );

        let future = ImmediateValuePromise::new(async move {
            let file = file.path.unwrap();
            let str_file = fs::read_to_string(file).unwrap();
            let mut parse_result = profile.parse_file(&str_file).unwrap();
            parse_result.rows.sort_by(ExpenseRecord::sorting_fn());

            let mut all_records_map = query
                .await
                .unwrap()
                .into_iter()
                .chunk_by(|rec| {
                    assert!(rec.origin().eq(&profile.origin_name));
                    (*rec.data_import(), rec.origin().clone())
                })
                .into_iter()
                .map(|(key, recs)| {
                    let mut records = recs.collect::<Vec<_>>();
                    records.sort_by(ExpenseRecord::sorting_fn());
                    (key, records)
                })
                .collect::<HashMap<_, _>>();

            let imported_rows = &parse_result.rows;
            let overlaps = all_records_map
                .iter_mut()
                .filter_map(|((uuid, origin), records)| {
                    assert!(origin.eq(imported_rows[0].origin()));

                    let mut first_match = None;
                    let match_count = *records
                        .iter()
                        .counts_by(|rec| {
                            for (index, imported_rec) in
                                imported_rows.iter().enumerate()
                            {
                                let is_same = imported_rec.is_same_record(rec);
                                if is_same {
                                    if first_match.is_none() {
                                        first_match = Some(index);
                                    }
                                    return true;
                                }
                            }
                            false
                        })
                        .get(&true)
                        .unwrap_or(&0);
                    match match_count {
                        0 => None,
                        _ => Some(ImportOverlap::new(
                            *uuid,
                            match_count,
                            records,
                            first_match.unwrap(),
                        )),
                    }
                })
                .collect::<Vec<_>>();

            Ok(ImportResultWithOverlap::new(parse_result, overlaps))
        });
        self.import_state.set_parsing(future);
    }

    fn save_parse(&mut self) {
        let ImportParsingState::Finished(import_result) = &self.import_state
        else {
            unreachable!();
        };

        let links = self.linker.find_links_from_new_records(
            import_result.rows.iter().map(|rec| &rec.1),
        );
        self.records.execute_many(|builder| {
            builder.execute(DbDataImport::insert(
                import_result.import.dml_clone(),
            ));
            builder.execute(DbRecord::insert_many(
                import_result.rows.iter().map(|rec| rec.1.dml_clone()),
            ));
            if !links.is_empty() {
                builder.execute(DbPossibleLink::insert_many(
                    links.into_iter().map(ToActiveModel::dml),
                ));
            }
        });
    }

    fn display_record_row(record: Option<&ExpenseRecord>, ui: &mut Ui) {
        let (amount, datetime, description) = record.map_or(
            (String::default(), String::default(), String::default()),
            |record| {
                (
                    record.formatted_amount(),
                    record.datetime().to_string(),
                    record.description().unwrap_or(&String::new()).clone(),
                )
            },
        );
        fn clamp_str(str: String) -> String {
            match str.len() <= 30 {
                true => str.clone(),
                false => str[0..30].to_string(),
            }
        }
        ui.label(clamp_str(amount));
        ui.label(clamp_str(datetime));
        ui.label(clamp_str(description));
    }
}

pub enum ImportParsingState {
    None,
    Parsing(ImmediateValuePromise<ImportResultWithOverlap>),
    Finished(ImportResultWithOverlap),
}

impl ImportParsingState {
    fn ready_for_new(&self) -> bool {
        matches!(self, ImportParsingState::None)
    }
    fn set_parsing(
        &mut self,
        future: ImmediateValuePromise<ImportResultWithOverlap>,
    ) {
        assert!(matches!(self, Self::None));
        let _ = mem::replace(self, ImportParsingState::Parsing(future));
    }
    fn try_resolve(&mut self) {
        let resolved_value = if let ImportParsingState::Parsing(promise) = self
        {
            if promise.poll_and_check_finished() {
                promise.take_value()
            } else {
                None
            }
        } else {
            None
        };

        if let Some(val) = resolved_value {
            let _ = mem::replace(self, Self::Finished(val));
        }
    }
    fn clear(&mut self) {
        let _ = mem::replace(self, ImportParsingState::None);
    }
}

pub struct ImportResultWithOverlap {
    rows: Vec<(bool, ExpenseRecord)>,
    import: DataImport,
    overlaps: Vec<ImportOverlap>,
    overlap_cleared: bool,
}

impl ImportResultWithOverlap {
    pub fn new(import: ParseResult, overlaps: Vec<ImportOverlap>) -> Self {
        Self {
            rows: import.rows.into_iter().map(|rec| (false, rec)).collect(),
            import: import.import,
            overlap_cleared: overlaps.is_empty(),
            overlaps,
        }
    }

    pub fn remove_overlap(&mut self, index: usize) {
        info!("this ran");
        self.overlaps.remove(index);
        self.overlap_cleared = self.overlaps.is_empty();
    }
}

pub struct ImportOverlap {
    existing_import: Uuid,
    count: usize,
    records: Vec<ExpenseRecord>,
    first_match: usize,
}

impl ImportOverlap {
    pub fn new(
        existing_import: Uuid,
        count: usize,
        records: &[ExpenseRecord],
        first_match: usize,
    ) -> Self {
        Self {
            existing_import,
            count,
            records: records.to_vec(),
            first_match,
        }
    }
}

const PARSED_RECORDS_EMPTY_TEXT: &str = r#"
Drop in some files, select a profile and then click on 'Parse Files' to preview the parsed records
before clicking on 'Save parsed Data' to save them.
"#;
