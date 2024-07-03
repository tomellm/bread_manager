use std::{cell::RefCell, ops::Deref, rc::Rc, sync::Arc};

use egui::{ProgressBar, Ui};
use lazy_async_promise::{
    api_macros::{send_data, set_error, set_finished, unpack_result},
    DataState, LazyVecPromise, Message, Promise,
};
use tokio::sync::{mpsc, watch};
use uuid::Uuid;

use crate::{
    model::profiles::{
        DateTimeColumn, ExpenseColumn, ExpenseDate, ExpenseDateTime, ExpenseTime,
        IntermediateParse, IntermediateProfileState, ParsableWrapper, Profile, ProfileBuilder,
    },
    utils::{changer::Response, communicator::Communicator, misc::clear_vec, ui_state::UiStates},
};

use super::super::utils::{drag_int, option_display, other_column_editor, single_char, text};

pub struct CreateProfile {
    reciver: mpsc::Receiver<egui::DroppedFile>,
    update_callback_ctx: Option<egui::Context>,
    testing_file: Arc<Option<egui::DroppedFile>>,
    parsed_testing_file: Option<LazyVecPromise<IntermediateParse>>,
    profile_builder: Arc<ProfileBuilder>,
    intermediate_profile_state: IntermediateProfileState,
    profiles_communicator: Communicator<Uuid, Profile>,
    ui_states: UiStates,
}

impl CreateProfile {
    pub fn new(
        reciver: mpsc::Receiver<egui::DroppedFile>,
        profiles_communicator: Communicator<Uuid, Profile>,
    ) -> Self {
        Self {
            reciver,
            update_callback_ctx: None,
            testing_file: Arc::new(None),
            parsed_testing_file: None,
            profile_builder: Arc::new(ProfileBuilder::default()),
            intermediate_profile_state: IntermediateProfileState::default(),
            profiles_communicator,
            ui_states: UiStates::default(),
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        self.update_callback_ctx = Some(ctx.clone());
        self.recive_files();

        ui.horizontal(|ui| {
            ui.heading("Create Profiles:");
            if ui.button("reset").clicked() {
                self.reset();
            }
        });
        self.show_editing_controls(ui);
        if ui.button("update builder").clicked() {
            self.update_builder();
            self.update_parse_test();
        }
        ui.add_sized(
            [ui.available_width(), ui.available_height() - 50.],
            |ui: &mut Ui| {
                if let Some(ref mut parsed_file) = &mut self.parsed_testing_file {
                    egui::ScrollArea::both()
                        .max_height(ui.available_height())
                        .show(ui, |ui| match parsed_file.poll_state() {
                            DataState::Uninitialized => ui.label("Updating post list"),
                            DataState::Error(msg) => {
                                ui.label(format!("Error occurred while fetching post-list: {msg}"))
                            }
                            DataState::Updating(_) | DataState::UpToDate => {
                                egui::Grid::new("parsed test file")
                                    .show(ui, |ui| Self::parsed_example_view(ui, parsed_file))
                                    .inner
                            }
                        })
                        .inner
                } else {
                    ui.label("Drop a testing file here to test your profile on!")
                }
            },
        );

        ui.add_sized([ui.available_width(), 50.], |ui: &mut Ui| {
            ui.centered_and_justified(|ui| {
                let save_profile = self.save_profile();
                self.ui_states
                    .timer::<_, _, Option<watch::Receiver<Response<Uuid, Profile>>>>(
                        "save_ui",
                        ui,
                        5,
                        None,
                        |ui, state, start_timer| match save_profile {
                            Ok(save_profile_action) => {
                                if ui.button("save profile").clicked() {
                                    let _ = state.insert(save_profile_action());
                                    start_timer();
                                };
                            }
                            Err(()) => {
                                ui.label("profile is incorrect and cannot be saved");
                            }
                        },
                        |ui, state, percentage_passed| {
                            let Some(reciver) = state else {
                                panic!(
                                    r#"

This state shouldn't happen. Maybe you forgot to set the reciver but decided to
start the timer. Check that the reciver is beeing set correctly.

                            "#
                                );
                            };
                            match &*reciver.borrow() {
                                Response::Loading => ui.label("setting of profile is loading"),
                                Response::Worked(_) => ui.label("setting profile worked"),
                                Response::Error(_, err) => {
                                    ui.label(format!("setting profile failed: {err}"))
                                }
                            };
                            ui.add(ProgressBar::new(percentage_passed).animate(true));
                        },
                    );
            })
            .response
        });
    }

    fn parsed_example_view(
        ui: &mut Ui,
        parsed_file: &mut LazyVecPromise<IntermediateParse>,
    ) -> egui::Response {
        let slice = parsed_file.as_slice();
        if !slice.is_empty() {
            if let IntermediateParse::RowsAndCols(row) = &slice[0] {
                ui.label("");
                row.iter().enumerate().for_each(|(i, _)| {
                    ui.label(format!("{i}"));
                });
                ui.end_row();
            }
        }
        for (index, line) in slice.iter().enumerate() {
            let inverse_index = slice.len() - 1 - index;
            ui.label(format!("{index}\t{inverse_index}"));
            match line {
                IntermediateParse::Rows(row) => {
                    ui.label(row);
                    ui.end_row();
                }
                IntermediateParse::RowsAndCols(row) => {
                    for col in row {
                        ui.label(col);
                    }
                    ui.end_row();
                }
                IntermediateParse::None => {
                    ui.label("Could not parse the testfile with the current profile");
                }
            }
        }
        ui.label("\nno more lines")
    }

    pub fn edit(&mut self, profile: &Profile) {
        self.reset();
        self.intermediate_profile_state = IntermediateProfileState::from_profile(profile);
    }

    fn reset(&mut self) {
        let callback = self.update_callback();
        self.testing_file = Arc::new(None);
        self.parsed_testing_file = None;
        self.profile_builder = Arc::new(ProfileBuilder::default());
        self.intermediate_profile_state = IntermediateProfileState::default();
        callback();
    }

    fn update_callback(&self) -> impl Fn() {
        let ctx = self.update_callback_ctx.clone().unwrap();
        move || ctx.request_repaint()
    }

    fn recive_files(&mut self) {
        while let Ok(file) = self.reciver.try_recv() {
            self.testing_file = Arc::new(Some(file));
            self.update_parse_test();
            self.update_callback()();
        }
    }

    pub fn show_file_viewer() -> bool {
        true
    }

    fn show_editing_controls(&mut self, ui: &mut egui::Ui) {
        let CreateProfile {
            intermediate_profile_state:
                IntermediateProfileState {
                    name,
                    margin_top,
                    margin_btm,
                    delimiter,
                    expense_col,
                    datetime_col,
                    other_cols,
                    default_tags,
                    origin_name
                },
            ..
        } = self;
        text(ui, name);
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label("Delimiter");
                    single_char(ui, delimiter);
                });
            });
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label("Margin Top");
                    drag_int(ui, margin_top);
                });
            });
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label("Margin Bottom");
                    drag_int(ui, margin_btm);
                });
            });
        });
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                if ui.button("add default tag").clicked() {
                    default_tags.push(String::new());
                }
            });
            ui.add_space(10.);
            let mut to_delete = vec![];
            ui.horizontal_wrapped(|ui| {
                for (index, tag) in default_tags.iter_mut().enumerate() {
                    ui.add_sized([100., 25.], |ui: &mut Ui| {
                        let res = ui.add(egui::TextEdit::singleline(tag));
                        if ui.button("remove").clicked() {
                            to_delete.push(index);
                        }
                        res
                    });
                }
            });
            //responsive_columns(ui, items, other_column_editor);
            clear_vec(to_delete, default_tags);
        });
        ui.add_space(10.);
        ui.separator();
        ui.add_space(10.);
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.vertical(|ui| Self::expense_col_selection(ui, expense_col));
            });
            ui.group(|ui| {
                ui.vertical(|ui| {
                    Self::datetime_col_selection(ui, datetime_col);
                });
            });
            ui.add(egui::TextEdit::singleline(origin_name));
        });
        ui.add_space(10.);
        ui.separator();
        ui.add_space(10.);
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                if ui.button("other").clicked() {
                    other_cols.push((0, ParsableWrapper::income()));
                }
            });
            ui.add_space(10.);
            let to_delete = Rc::new(RefCell::new(vec![] as Vec<usize>));
            ui.horizontal_wrapped(|ui| {
                for (index, (ref mut col_pos, ref mut col_type)) in
                    other_cols.iter_mut().enumerate()
                {
                    ui.add_sized([175., 175.], |ui: &mut Ui| {
                        other_column_editor(ui, index, col_pos, col_type, &to_delete)
                    });
                }
            });
            //responsive_columns(ui, items, other_column_editor);
            clear_vec(to_delete.take(), other_cols);
        });
        ui.add_space(10.);
    }

    fn expense_col_selection(ui: &mut Ui, expense_col: &mut Option<ExpenseColumn>) {
        ui.label("Select the main expense column/s");
        ui.horizontal(|ui| {
            if let Some(expense) = expense_col {
                match expense {
                    ExpenseColumn::Split((pos1, _), (pos2, _)) => {
                        drag_int(ui, pos1);
                        drag_int(ui, pos2);
                    }
                    ExpenseColumn::Combined(pos, _) | ExpenseColumn::OnlyExpense(pos, _) => {
                        drag_int(ui, pos);
                    }
                }
            }
        });
        egui::ComboBox::from_label("expense")
            .selected_text(
                expense_col
                    .as_ref()
                    .map_or_else(|| String::from("Nothing"), |v| format!("{v}")),
            )
            .show_ui(ui, |ui| {
                ui.style_mut().wrap = Some(false);
                ui.set_min_width(60.0);
                ui.selectable_value(expense_col, None, "not yet");
                ui.selectable_value(expense_col, Some(ExpenseColumn::split(0, 0)), "Split");
                ui.selectable_value(expense_col, Some(ExpenseColumn::combined(0)), "Combined");
                ui.selectable_value(
                    expense_col,
                    Some(ExpenseColumn::only_expense(0)),
                    "Only Expense",
                );
            });
    }

    fn datetime_col_selection(ui: &mut Ui, datetime_col: &mut Option<DateTimeColumn>) {
        ui.horizontal(|ui| {
            if let Some(datetime) = datetime_col {
                match datetime {
                    DateTimeColumn::DateAndTime(
                        (pos1, ExpenseDate(format1)),
                        (pos2, ExpenseTime(format2)),
                    ) => {
                        ui.vertical(|ui| {
                            drag_int(ui, pos1);
                            drag_int(ui, pos2);
                        });
                        ui.vertical(|ui| {
                            text(ui, format1);
                            text(ui, format2);
                        });
                    }
                    DateTimeColumn::DateTime(pos, ExpenseDateTime(format))
                    | DateTimeColumn::Date(pos, ExpenseDate(format)) => {
                        drag_int(ui, pos);
                        text(ui, format);
                    }
                }
            }
        });
        egui::ComboBox::from_label("datetime")
            .selected_text(option_display(datetime_col.as_ref()))
            .show_ui(ui, |ui| {
                ui.style_mut().wrap = Some(false);
                ui.set_min_width(60.0);
                ui.selectable_value(datetime_col, None, "not yet");
                ui.selectable_value(datetime_col, Some(DateTimeColumn::new_date()), "Date");
                ui.selectable_value(
                    datetime_col,
                    Some(DateTimeColumn::new_datetime()),
                    "DateTime",
                );
                ui.selectable_value(
                    datetime_col,
                    Some(DateTimeColumn::new_date_time()),
                    "DateAndTime",
                );
            });
    }

    fn update_parse_test(&mut self) {
        if self.testing_file.is_none() {
            self.parsed_testing_file = None;
            return;
        };
        let to_be_parsed = Arc::clone(&self.testing_file);
        let builder = Arc::clone(&self.profile_builder);
        println!("{builder:?}");
        let updater = move |tx: mpsc::Sender<Message<IntermediateParse>>| {
            let to_be_parsed = Arc::clone(&to_be_parsed);
            let builder = Arc::clone(&builder);
            async move {
                let file = to_be_parsed.deref().clone();
                let file = file.unwrap().path.unwrap();
                let str_file = unpack_result!(std::fs::read_to_string(file), tx);
                let rows: Vec<String> = str_file.lines().map(str::to_string).collect();
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

    fn update_builder(&mut self) {
        self.profile_builder = Arc::new(ProfileBuilder::from_inter_state(
            &self.intermediate_profile_state,
        ));
    }

    fn save_profile(
        &mut self,
    ) -> Result<impl FnOnce() -> watch::Receiver<Response<Uuid, Profile>>, ()> {
        self.update_builder();
        let profile = (*self.profile_builder).clone().build().map_err(|()| {})?;
        let mut setter = self.profiles_communicator.set_action();
        Ok(move || setter(profile))
    }
}
