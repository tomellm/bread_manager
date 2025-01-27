use std::fs;

use egui::Ui;
use egui_light_states::{future_await::FutureAwait, UiStates};
use hermes::{
    carrier::{execute::ImplExecuteCarrier, query::ImplQueryCarrier},
    container::projecting::ProjectingContainer,
    factory::Factory,
    ToActiveModel,
};
use lazy_async_promise::{DirectCacheAccess, ImmediateValuePromise, ImmediateValueState};
use sea_orm::{EntityOrSelect, EntityTrait};
use tracing::warn;

use crate::{
    components::expense_records::table::RecordsTable,
    db::{possible_links::DbPossibleLink, records::DbRecord},
    model::{
        linker::{Linker, PossibleLink},
        records::ExpenseRecord,
    },
};

use super::files_to_parse::FileToParse;

pub(super) struct ParsedRecords {
    records: ProjectingContainer<ExpenseRecord, DbRecord>,
    possible_links: ProjectingContainer<PossibleLink, DbPossibleLink>,
    parsed_records: Vec<ExpenseRecord>,
    futures: Vec<ImmediateValuePromise<Vec<ExpenseRecord>>>,
    linker: Linker,
    ui: UiStates,
    table: RecordsTable,
}

impl ParsedRecords {
    pub(super) fn init(
        factory: Factory,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            let mut records = factory.builder().projector();
            let mut possible_links = factory.builder().projector();

            let _ = records.query(DbRecord::find().select());
            let _ = possible_links.query(DbPossibleLink::find().select());

            Self {
                records,
                possible_links,
                linker: Linker::init(factory).await,
                parsed_records: vec![],
                futures: vec![],
                ui: UiStates::default(),
                table: RecordsTable::default(),
            }
        }
    }

    pub fn ui_update(&mut self, ui: &mut Ui) {
        self.state_update();
        self.records.state_update();
        self.possible_links.state_update();
        self.linker.state_update();

        ui.heading("Parsed Data:");
        if !self.parsed_records.is_empty() || self.ui.is_running::<()>("save_parsed_data") {
            ui.vertical_centered(|ui| {
                if ui.button("Save parsed Data").clicked() {
                    let future = self.save_parsed_data();
                    // ToDo add response
                    //self.ui.set_future("save_parsed_data").set(future);
                }
            });
        }
        if !self.parsed_records.is_empty() {
            self.table.toggles(ui);
            self.table.show(&self.parsed_records, ui);
        } else {
            ui.vertical_centered_justified(|ui| {
                ui.add_space(30.);
                ui.label(PARSED_RECORDS_EMPTY_TEXT);
                ui.add_space(30.);
            });
        }
    }

    fn create_future(file: FileToParse) -> ImmediateValuePromise<Vec<ExpenseRecord>> {
        ImmediateValuePromise::new(async move {
            let FileToParse {
                file,
                profile: Some(profile),
                ..
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
    pub fn parse_files(&mut self, files: impl Iterator<Item = FileToParse>) {
        let futures = files.map(Self::create_future);
        self.futures.extend(futures);
    }

    pub fn handle_parsed_records(&mut self, new_records: Vec<ExpenseRecord>) {
        self.parsed_records.extend(new_records);
    }

    pub fn state_update(&mut self) {
        let resulting_expenses = self
            .futures
            .extract_if(.., |working_future| {
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

    pub fn save_parsed_data(&mut self) {
        let records = self.parsed_records.drain(..).collect::<Vec<_>>();
        let links = self.linker.find_links(&records);

        self.records.execute_many(|builder| {
            builder
                .execute(DbRecord::insert_many(
                    records.into_iter().map(ToActiveModel::dml),
                ))
                .execute(DbPossibleLink::insert_many(
                    links.into_iter().map(ToActiveModel::dml),
                ));
        });
    }
}

const PARSED_RECORDS_EMPTY_TEXT: &str = r#"
Drop in some files, select a profile and then click on 'Parse Files' to preview the parsed records
before clicking on 'Save parsed Data' to save them.
"#;
