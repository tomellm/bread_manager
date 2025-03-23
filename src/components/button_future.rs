use egui::Ui;
use egui_light_states::{future_await::FutureAwait, UiStates};
use lazy_async_promise::ImmediateValuePromise;

pub trait ButtonWithFuture {
    fn button_future<'a, P, T>(
        &mut self,
        name: &str,
        ui_state: &mut UiStates,
        promise_producer: impl (FnOnce() -> P) + 'a,
    ) where
        P: Into<ImmediateValuePromise<T>>,
        T: Send + 'static;
}

impl ButtonWithFuture for Ui {
    fn button_future<'a, P, T>(
        &mut self,
        name: &str,
        ui_state: &mut UiStates,
        promise_producer: impl (FnOnce() -> P) + 'a,
    ) where
        P: Into<ImmediateValuePromise<T>>,
        T: Send + 'static,
    {
        let future_name = format!("{name}_future_await");
        self.add_enabled_ui(!ui_state.is_running::<T>(&future_name), |ui| {
            if ui.button(name).clicked() {
                ui_state.set_future(&future_name).set(promise_producer());
            }
        });
        ui_state
            .future_status::<T>(future_name)
            .default()
            .show(self);
    }
}
