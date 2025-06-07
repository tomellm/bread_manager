use egui::{
    popup_below_widget, Color32, Grid, Id, Label, PopupCloseBehavior, RichText,
    Ui, Widget,
};

use super::FileToParse;

pub fn margin_cutoff(file: &FileToParse, ui: &mut Ui) {
    let Some(profile) = &file.profile else {
        ui.label("select a profile");
        return;
    };

    let margin_show_response = ui.button("show");
    let popup_id = Id::new(format!("popup_id_{}", file.uuid));
    if margin_show_response.clicked() {
        ui.memory_mut(|mem| mem.toggle_popup(popup_id));
    }
    popup_below_widget(
        ui,
        popup_id,
        &margin_show_response,
        PopupCloseBehavior::CloseOnClickOutside,
        |ui| {
            if !file.cut_off_margins.is_set() {
                ui.label("Nothing will be cut off.");
                return;
            }

            ui.label(format!(
                "Cut off margin for top is: {} and for the bottom {}.",
                profile.margins.0, profile.margins.1
            ));
            ui.label("Note that only the brigther text will be removed not the dark one");
            ui.separator();

            let luminance = 70;

            Grid::new(format!("cut_off_margis_grid_{}", file.uuid)).show(
                ui,
                |ui| {
                    if let Some(top) = file.cut_off_margins.top.as_ref() {
                        for (index, row) in
                            top[0..(top.len() - 1)].iter().enumerate()
                        {
                            ui.label((index + 1).to_string());
                            for el in row {
                                ui.label(el);
                            }
                            ui.end_row();
                        }

                        ui.label("");
                        for el in top.last().unwrap() {
                            Label::new(
                                RichText::new(el)
                                    .color(Color32::from_gray(luminance)),
                            )
                            .ui(ui);
                        }
                        ui.end_row();
                    }

                    ui.label("");
                    for _ in 0..file.cut_off_margins.width().unwrap() {
                        Label::new(
                            RichText::new("...")
                                .color(Color32::from_gray(luminance)),
                        )
                        .ui(ui);
                    }
                    ui.end_row();

                    if let Some(bottom) = file.cut_off_margins.bottom.as_ref() {
                        ui.label("");
                        for el in bottom.first().unwrap() {
                            Label::new(
                                RichText::new(el)
                                    .color(Color32::from_gray(luminance)),
                            )
                            .ui(ui);
                        }
                        ui.end_row();

                        for (index, row) in
                            bottom[1..bottom.len()].iter().enumerate()
                        {
                            ui.label((profile.margins.1 - index).to_string());
                            for el in row {
                                ui.label(el);
                            }
                            ui.end_row();
                        }
                    }
                },
            );
        },
    );
}

#[derive(Clone, Debug, Default)]
pub struct CutOffMargins {
    pub top: Option<Vec<Vec<String>>>,
    pub bottom: Option<Vec<Vec<String>>>,
}

impl CutOffMargins {
    const MAX_STR_WIDTH: usize = 30;
    pub fn push_top(&mut self, str: &str, split: char) {
        let top = self.top.get_or_insert_with(Vec::default);
        top.push(
            str.split(split)
                .map(|str| {
                    if str.len() < Self::MAX_STR_WIDTH {
                        str.to_owned()
                    } else {
                        format!(
                            "{}...",
                            &str[0..Self::MAX_STR_WIDTH].replace(" ", "")
                        )
                    }
                })
                .collect(),
        );
    }
    pub fn push_bottom(&mut self, str: &str, split: char) {
        let bottom = self.bottom.get_or_insert_with(Vec::default);
        bottom.push(
            str.split(split)
                .map(|str| {
                    if str.len() < Self::MAX_STR_WIDTH {
                        str.to_owned()
                    } else {
                        format!(
                            "{}...",
                            &str[0..Self::MAX_STR_WIDTH].replace(" ", "")
                        )
                    }
                })
                .collect(),
        );
    }

    pub fn clear(&mut self) {
        self.top = None;
        self.bottom = None;
    }

    fn is_set(&self) -> bool {
        self.top.is_some() || self.bottom.is_some()
    }

    fn width(&self) -> Option<usize> {
        match (&self.top, &self.bottom) {
            (Some(vec), None) | (_, Some(vec)) => {
                Some(vec.first().unwrap().len())
            }
            _ => None,
        }
    }
}
