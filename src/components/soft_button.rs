use std::hash::Hash;

use egui::{Frame, Label, Response, RichText, Ui, UiBuilder, Widget};

pub fn soft_button(
    id_salt: impl Hash,
    text: impl Into<String>,
    ui: &mut Ui,
    ) -> Response {
    ui.scope_builder(
        UiBuilder::new()
            .id_salt(id_salt)
            .sense(egui::Sense::click()),
        |ui| {
            let response = ui.response();
            let visuals = ui.style().interact(&response);
            let text_color = visuals.text_color();

            Frame::canvas(ui.style())
                .fill(visuals.bg_fill.gamma_multiply(0.3))
                .stroke(visuals.bg_stroke)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        Label::new(RichText::new(text).color(text_color))
                            .selectable(false)
                            .ui(ui);
                    });
                });
        },
    )
    .response
}
