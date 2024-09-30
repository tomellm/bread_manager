use std::{future::Future, mem};

use eframe::App;
use egui::Spinner;
use lazy_async_promise::{DirectCacheAccess, ImmediateValuePromise, ImmediateValueState};

pub struct LoadingScreen<T>
where
    T: App + Send + 'static,
{
    pub(super) state: State<T>,
}

impl<T> LoadingScreen<T>
where
    T: App + Send + 'static,
{
    pub fn try_resolve(&mut self) {
        let State::Loading(ref mut promise) = self.state else {
            unreachable!()
        };
        let new_state = match promise.take_result() {
            None | Some(Err(_)) => State::Error("Error".into()),
            Some(Ok(value)) => State::Finished(value),
        };
        let _ = mem::replace(&mut self.state, new_state);
    }
}

impl<F, T> From<F> for LoadingScreen<T>
where
    F: Future<Output = T> + Send + 'static,
    T: App + Send + 'static,
{
    fn from(value: F) -> Self {
        let promise = ImmediateValuePromise::new(async move {
            Ok(value.await)
        });
        Self {
            state: promise.into()
        }
        
    }
}

impl<T> App for LoadingScreen<T>
where
    T: App + Send + 'static,
{
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let State::Finished(ref mut app) = self.state {
            app.update(ctx, frame);
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.add_sized(ui.available_size(), |ui: &mut egui::Ui| {
                    let mut finished = false;
                    let response = match &mut self.state {
                        State::Finished(_) => ui.label("Done!"),
                        State::Loading(promise) => {
                            if !matches!(promise.poll_state(), ImmediateValueState::Updating) {
                                finished = true;
                            }
                            ui.add(Spinner::new())
                        }
                        State::Error(err) => ui.label(err.as_str()),
                    };

                    if finished {
                        self.try_resolve();
                    }
                    response
                });
            });
        }
    }
}

pub(super) enum State<T>
where
    T: App + Send + 'static,
{
    Loading(ImmediateValuePromise<T>),
    Finished(T),
    Error(String),
}

impl<T> From<ImmediateValuePromise<T>> for State<T>
where
    T: App + Send + 'static,
{
    fn from(value: ImmediateValuePromise<T>) -> Self {
        Self::Loading(value)
    }
}
