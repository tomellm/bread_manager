use std::{mem, sync::Arc};

use eframe::App;
use egui::{CentralPanel, Ui};
use hermes::{
    carrier::query::ImplQueryCarrier,
    container::{data::ImplData, projecting::ProjectingContainer},
    factory::Factory,
};
use itertools::Itertools;
use lazy_async_promise::{DirectCacheAccess, ImmediateValuePromise};
use sea_orm::{EntityOrSelect, EntityTrait};
use uuid::Uuid;

use crate::{
    components::expense_records::full_view::ExpenseRecordFullView,
    db::records::DbRecord,
    model::records::{ExpenseRecord, ExpenseRecordUuid},
    utils::PromiseUtilities,
};

pub struct RecordView {
    records: ProjectingContainer<ExpenseRecord, DbRecord>,
    search_context: SearchContext,
    current_screen: RecordScreen,
}

impl RecordView {
    pub fn init(
        factory: Factory,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            let mut records =
                factory.builder().name("record_view_records").projector();
            records.stored_query(DbRecord::find().select());
            Self {
                records,
                search_context: SearchContext::default(),
                current_screen: RecordScreen::default(),
            }
        }
    }

    fn state_update(&mut self) {
        self.records.state_update(true);
        self.search_context.state_update();

        if let Some(result) = self.search_context.result.take() {
            match result {
                SearchResult {
                    record: Some(record),
                    ..
                } => self.current_screen.record(record),
                SearchResult {
                    search_error: Some(error),
                    ..
                } => self.current_screen.error(error),
                _ => (),
            }
        }
    }

    fn screen_ui(&mut self, ui: &mut Ui) {
        match &mut self.current_screen {
            RecordScreen::Empty => {
                ui.label("Nothing to see yet, search something to get started");
            }
            RecordScreen::RecordView(expense_record_full_view) => {
                ui.add(expense_record_full_view.as_mut());
            }
            RecordScreen::Error(ref error) => {
                ui.heading("Error:");
                ui.label(error);
            }
        }
    }
}

impl App for RecordView {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.state_update();

        CentralPanel::default().show(ctx, |ui| {
            ui.label(format!("{}", self.records.data().len()));
            ui.horizontal(|ui| {
                ui.text_edit_singleline(
                    &mut self.search_context.parameters.uuid_search,
                );
                if ui.button("search").clicked() {
                    self.search_context.find_first(&self.records);
                }
            });
            self.screen_ui(ui);
        });
    }
}

#[derive(Default)]
pub enum RecordScreen {
    #[default]
    Empty,
    RecordView(Box<ExpenseRecordFullView>),
    Error(String),
}

impl RecordScreen {
    pub fn record(&mut self, record: ExpenseRecord) {
        let _ = mem::replace(
            self,
            Self::RecordView(Box::new(ExpenseRecordFullView::new(record))),
        );
    }
    pub fn error(&mut self, error: String) {
        let _ = mem::replace(self, Self::Error(error));
    }
}

#[derive(Default)]
struct SearchContext {
    parameters: SearchParameters,
    search_future: Option<ImmediateValuePromise<SearchResult>>,
    result: Option<SearchResult>,
}

impl SearchContext {
    pub fn state_update(&mut self) {
        if let Some(mut future) = self.search_future.take() {
            if future.poll_and_check_finished() {
                let Ok(result) = future.take_result().unwrap() else {
                    unreachable!()
                };
                let _ = self.result.insert(result);
            } else {
                let _ = self.search_future.insert(future);
            }
        }
    }

    pub fn find_first(&mut self, data: &impl ImplData<ExpenseRecord>) {
        let data = Arc::clone(data.data());
        let parameters = self.parameters.clone();

        let future = async move {
            let Ok(uuid) = Uuid::parse_str(parameters.uuid_search())
                .map(ExpenseRecordUuid::from)
            else {
                return SearchResult::error("text is not uuid");
            };
            let remaining = data
                .iter()
                .filter(|record| uuid.eq(record.uuid()))
                .collect_vec();

            match remaining.len() {
                0 => SearchResult::error("there are no results with this uuid"),
                1 => SearchResult::new((*remaining.first().unwrap()).clone()),
                _ => SearchResult::error(
                    "there is more then one record with this uuid",
                ),
            }
        };
        let _ = self.search_future.insert(future.into());
    }

    pub fn set_result(&mut self, record: ExpenseRecord) {
        let _ = self.result.insert(SearchResult::new(record));
    }

    pub fn set_future(&mut self, future: ImmediateValuePromise<SearchResult>) {
        let _ = self.search_future.insert(future);
    }

    pub fn result(&self) -> Option<&ExpenseRecord> {
        self.result
            .as_ref()
            .and_then(|result| result.record.as_ref())
    }
}

#[derive(Clone, Default)]
struct SearchParameters {
    uuid_search: String,
}

impl SearchParameters {
    fn uuid_search(&self) -> &str {
        self.uuid_search.trim()
    }
}

#[derive(Default)]
struct SearchResult {
    record: Option<ExpenseRecord>,
    search_error: Option<String>,
}

impl SearchResult {
    fn new(record: ExpenseRecord) -> Self {
        Self {
            record: Some(record),
            search_error: None,
        }
    }

    fn error(error: impl Into<String>) -> Self {
        Self {
            record: None,
            search_error: Some(error.into()),
        }
    }

    fn none() -> Self {
        Self::default()
    }
}
