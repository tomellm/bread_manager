use std::{ops::Deref, sync::Arc};

use egui::{DroppedFile, Response, Ui};
use lazy_async_promise::{
    send_data, set_error, set_finished, unpack_result, DataState,
    LazyVecPromise, Message, Promise,
};
use tokio::sync::mpsc;
use tracing::trace;

use crate::{
    apps::utils::blank_option_display,
    model::profiles::builder::{IntermediateParse, ProfileBuilder},
    utils::{CompressDisplayResult, CompressResult},
};

pub(super) struct ProfilePreview {
    reciver: mpsc::Receiver<DroppedFile>,
    testing_file: Arc<Option<DroppedFile>>,
    parsed_testing_file: Option<LazyVecPromise<IntermediateParse>>,
}

impl ProfilePreview {
    pub fn new(reciver: mpsc::Receiver<egui::DroppedFile>) -> Self {
        Self {
            reciver,
            testing_file: Arc::new(None),
            parsed_testing_file: None,
        }
    }

    pub fn profile_preview(
        &mut self,
        ui: &mut Ui,
        builder: &Arc<ProfileBuilder>,
    ) -> Response {
        self.recive_files(builder);

        if let Some(ref mut parsed_file) = &mut self.parsed_testing_file {
            egui::ScrollArea::both()
                .show(ui, |ui| match parsed_file.poll_state() {
                    DataState::Uninitialized => ui.label("Updating post list"),
                    DataState::Error(msg) => {
                        ui.label(format!("Error occurred while fetching post-list: {msg}"))
                    }
                    DataState::Updating(_) | DataState::UpToDate => {
                        egui::Grid::new("parsed test file")
                            .show(ui, |ui| {
                                let slice = parsed_file.as_slice();
                                if !slice.is_empty() {
                                    if let IntermediateParse::RowsAndCols(row) = &slice[0] {
                                        ui.label("");
                                        row.iter().enumerate().for_each(|(i, _)| {
                                            ui.label(format!(
                                                "{i} {}",
                                                blank_option_display(builder.get_from_pos(i).as_ref())
                                            ));
                                        });
                                        ui.end_row();
                                    }
                                }
                                for (index, line) in slice.iter().enumerate() {
                                    let inverse_index = slice.len() - 1 - index;
                                    ui.label(format!("{index}\t{inverse_index}"));
                                    match line {
                                        IntermediateParse::Rows(row) => {
                                            ui.label(row.compress());
                                            ui.end_row();
                                        }
                                        IntermediateParse::RowsAndCols(row) => {
                                            for col in row {
                                                ui.label(col.compless_display());
                                            }
                                            ui.end_row();
                                        }
                                        IntermediateParse::None => {
                                            ui.label(
                                        "Could not parse the testfile with the current profile",
                                    );
                                        }
                                    }
                                }
                                ui.label("\nno more lines")
                            })
                            .inner
                    }
                })
                .inner
        } else {
            ui.label("Drop a testing file here to test your profile on!")
        }
    }

    pub fn reset(&mut self) {
        self.testing_file = Arc::new(None);
        self.parsed_testing_file = None;
    }

    fn recive_files(&mut self, builder: &Arc<ProfileBuilder>) {
        while let Ok(file) = self.reciver.try_recv() {
            self.testing_file = Arc::new(Some(file));
            self.update_parse_test(builder);
        }
    }

    pub fn update_parse_test(&mut self, builder: &Arc<ProfileBuilder>) {
        if self.testing_file.is_none() {
            self.parsed_testing_file = None;
            return;
        };
        let to_be_parsed = Arc::clone(&self.testing_file);
        let builder = Arc::clone(builder);
        trace!(msg = format!("{builder:?}"));
        let updater = move |tx: mpsc::Sender<Message<IntermediateParse>>| {
            let to_be_parsed = Arc::clone(&to_be_parsed);
            let builder = Arc::clone(&builder);
            async move {
                let file = to_be_parsed.deref().clone();
                let file = file.unwrap().path.unwrap();
                let str_file =
                    unpack_result!(std::fs::read_to_string(file), tx);
                let rows: Vec<String> =
                    str_file.lines().map(str::to_string).collect();
                let total_len = rows.len();
                for (index, row) in rows.into_iter().enumerate() {
                    let parsed = unpack_result!(
                        builder
                            .intermediate_parse(index, row, total_len)
                            .or(Err("")),
                        tx
                    );
                    if parsed != IntermediateParse::None {
                        send_data!(parsed, tx);
                    }
                }
                set_finished!(tx);
            }
        };

        self.parsed_testing_file = Some(LazyVecPromise::new(updater, 1));
    }
}
