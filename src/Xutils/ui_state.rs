use core::panic;
use std::{any::Any, collections::HashMap};

use chrono::{DateTime, Duration, Local};
use egui::Ui;
use tokio::sync::watch;

use super::{changer::Response, communicator::GetKey};

#[derive(Default)]
pub struct UiStates {
    map: HashMap<String, UiStateType>,
}

impl UiStates {
    pub fn timer<DoneUi, TimingUi, State>(
        &mut self,
        name: &str,
        ui: &mut Ui,
        seconds: i64,
        init_state: State,
        new_timer_done_ui: DoneUi,
        new_timer_timing_ui: TimingUi,
    ) where
        DoneUi: FnOnce(&mut Ui, &mut State, &mut dyn FnMut<(), Output = ()>) + 'static,
        TimingUi: FnOnce(&mut Ui, &mut State, f32) + 'static,
        State: InternalState + 'static,
    {
        let name = String::from(name);
        let timer_state = if let Some(timer_state) = self.map.get_mut(&name) {
            match timer_state {
                UiStateType::TimerState {
                    timer_done_ui,
                    timer_timing_ui,
                    ..
                } => {
                    let _ = timer_done_ui.insert(Box::new(
                        |ui: &mut Ui,
                         state: &mut Box<dyn InternalState>,
                         reset_fn: &mut dyn FnMut<(), Output = ()>| {
                            new_timer_done_ui(
                                ui,
                                state.to().downcast_mut::<State>().unwrap(),
                                reset_fn,
                            );
                        },
                    ));
                    let _ = timer_timing_ui.insert(Box::new(
                        |ui: &mut Ui,
                         state: &mut Box<dyn InternalState>,
                         percentage_passed: f32| {
                            new_timer_timing_ui(
                                ui,
                                state.to().downcast_mut::<State>().unwrap(),
                                percentage_passed,
                            );
                        },
                    ));
                }
                #[allow(unreachable_patterns)]
                _ => panic!(
                    r#"
                    
The `timer` function of `UiStates` was called with a specific name but this name
is already contained in the hashmap and is not of the correct `UiStateType`
leading to an unexpected situation.

Please make sure that you are not using the same name for different ui states.

                    "#
                ),
            }
            timer_state
        } else {
            self.map.insert(
                name.clone(),
                UiStateType::timer(
                    seconds,
                    init_state,
                    Box::new(
                        |ui: &mut Ui,
                         state: &mut Box<dyn InternalState>,
                         reset_fn: &mut dyn FnMut<(), Output = ()>| {
                            new_timer_done_ui(
                                ui,
                                state.to().downcast_mut::<State>().unwrap(),
                                reset_fn,
                            );
                        },
                    ),
                    Box::new(
                        |ui: &mut Ui,
                         state: &mut Box<dyn InternalState>,
                         percentage_passed: f32| {
                            new_timer_timing_ui(
                                ui,
                                state.to().downcast_mut::<State>().unwrap(),
                                percentage_passed,
                            );
                        },
                    ),
                ),
            );
            self.map.get_mut(&name).unwrap()
        };
        timer_state.ui::<State>(ui);
    }
}

pub trait InternalState: 'static {
    fn to(&mut self) -> &mut dyn Any;
}

impl<Key, Value> InternalState for Option<watch::Receiver<Response<Key, Value>>>
where
    Key: Clone + Send + Sync + 'static,
    Value: GetKey<Key> + Clone + Send + Sync + 'static,
{
    fn to(&mut self) -> &mut dyn Any {
        self
    }
}

enum UiStateType {
    TimerState {
        timer_started: Option<DateTime<Local>>,
        timer_duration: Duration,
        state: Box<dyn InternalState>,
        timer_done_ui: Option<TimerDoneUi>,
        timer_timing_ui: Option<TimerTimingUi>,
    },
}
type TimerDoneUi =
    Box<dyn FnOnce(&mut Ui, &mut Box<dyn InternalState>, &mut dyn FnMut<(), Output = ()>)>;
type TimerTimingUi = Box<dyn FnOnce(&mut Ui, &mut Box<dyn InternalState>, f32)>;

impl UiStateType {
    pub fn timer<State>(
        seconds: i64,
        state: State,
        timer_done_ui: TimerDoneUi,
        timer_timing_ui: TimerTimingUi,
    ) -> Self
    where
        State: InternalState + 'static,
    {
        Self::TimerState {
            timer_started: None,
            timer_duration: Duration::seconds(seconds),
            timer_done_ui: Some(timer_done_ui),
            timer_timing_ui: Some(timer_timing_ui),
            state: Box::new(state),
        }
    }

    pub fn ui<State>(&mut self, ui: &mut Ui)
    where
        State: InternalState + 'static,
    {
        match self {
            UiStateType::TimerState {
                timer_started,
                timer_duration,
                state,
                timer_done_ui,
                timer_timing_ui,
            } => match timer_started.as_ref() {
                None => {
                    let mut reset_timer = || {
                        let _ = timer_started.insert(Local::now());
                    };
                    if let Some(timer_done_ui_fn) = timer_done_ui.take() {
                        timer_done_ui_fn(ui, state, &mut reset_timer);
                    }
                }
                Some(start_time) => {
                    let now = Local::now();
                    let passed_time = now - *start_time;

                    if let Some(timer_timing_ui_fn) = timer_timing_ui.take() {
                        let percentage_passed = passed_time.num_milliseconds() as f64
                            / timer_duration.num_milliseconds() as f64;

                        timer_timing_ui_fn(ui, state, percentage_passed as f32);
                    }

                    if passed_time >= *timer_duration {
                        let _ = timer_started.take();
                    }
                }
            },
        }
    }
}
