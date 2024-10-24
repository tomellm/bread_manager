use std::fs;

use data_communicator::buffered::{communicator::Communicator, query::QueryType};
use egui::{Grid, ScrollArea, Ui};
use egui_light_states::{future_await::FutureAwait, UiStates};
use lazy_async_promise::{DirectCacheAccess, ImmediateValuePromise, ImmediateValueState};
use tracing::warn;
use uuid::Uuid;

use crate::{
    components::expense_records::table::RecordsTable, db::possible_links, model::{
        linker::{Link, Linker, PossibleLink},
        records::ExpenseRecord,
    }
};

use super::files_to_parse::FileToParse;

pub(super) struct ParsedRecords {
    records: Communicator<Uuid, ExpenseRecord>,
    possible_links: Communicator<Uuid, PossibleLink>,
    parsed_records: Vec<ExpenseRecord>,
    futures: Vec<ImmediateValuePromise<Vec<ExpenseRecord>>>,
    linker: Linker,
    ui: UiStates,
    table: RecordsTable,
}

impl ParsedRecords {
    pub(super) fn init(
        [records_one, records_two]: [Communicator<Uuid, ExpenseRecord>; 2],
        [possible_links_one, possible_links_two]: [Communicator<Uuid, PossibleLink>; 2],
        links: Communicator<Uuid, Link>,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            let _ = records_one.query(QueryType::All).await;
            Self {
                records: records_one,
                possible_links: possible_links_one,
                linker: Linker::init(possible_links_two, links, records_two).await,
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
                    self.ui.set_future("save_parsed_data").set(future);
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
            .extract_if(|working_future| {
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

    pub fn save_parsed_data(&mut self) -> ImmediateValuePromise<()> {
        let records = self.parsed_records.drain(..).collect::<Vec<_>>();
        let links = self.linker.find_links(&records);

        let records_future = self.records.insert_many(records);
        let links_future = self.possible_links.insert_many(links);
        async move {
            let _ = records_future.await;
            let _ = links_future.await;
        }
        .into()
    }
}

const PARSED_RECORDS_EMPTY_TEXT: &str = r#"
Drop in some files, select a profile and then click on 'Parse Files' to preview the parsed records
before clicking on 'Save parsed Data' to save them.
"#;
