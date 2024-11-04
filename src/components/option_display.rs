use egui::{Ui, WidgetText};

pub trait OptionDisplay<T> {
    fn display<'a>(
        &'a self,
        present_ui: impl FnOnce(&mut Ui, &T) + 'a,
        missing_ui: impl FnOnce(&mut Ui) + 'a,
    ) -> OptionDisplayBuilder<'a, T>;
    fn text_display<'a>(
        &'a self,
        present_ui: impl FnOnce(&mut Ui, &T) + 'a,
        missing_text: impl Into<WidgetText> + 'a,
    ) -> OptionDisplayBuilder<'a, T>; 
}

impl<T> OptionDisplay<T> for Option<T> {
    fn display<'a>(
            &'a self,
            present_ui: impl FnOnce(&mut Ui, &T) + 'a,
            missing_ui: impl FnOnce(&mut Ui) + 'a,
        ) -> OptionDisplayBuilder<'a, T> {
        OptionDisplayBuilder {
            option: self,
            present_ui: Box::new(present_ui),
            missing_ui: Box::new(missing_ui),
        }
    }
    fn text_display<'a>(
            &'a self,
            present_ui: impl FnOnce(&mut Ui, &T) + 'a,
            missing_text: impl Into<WidgetText> + 'a,
        ) -> OptionDisplayBuilder<'a, T> {
        OptionDisplayBuilder {
            option: self,
            present_ui: Box::new(present_ui),
            missing_ui: Box::new(|ui: &mut Ui| {
                ui.label(missing_text);
            }),
        }   
    }
}

pub trait AutoOptionDisplay<T> {
    fn auto_display<'a>(
        &'a self,
        missing_text: impl Into<WidgetText> + 'a,
    ) -> OptionDisplayBuilder<'a, T>; 
}

impl<T> AutoOptionDisplay<T> for Option<T>
where
    T: Into<WidgetText> + Clone
{
    fn auto_display<'a>(
            &'a self,
            missing_text: impl Into<WidgetText> + 'a,
        ) -> OptionDisplayBuilder<'a, T> {
        OptionDisplayBuilder {
            option: self,
            present_ui: Box::new(|ui: &mut Ui, val: &T| {
                ui.label(val.clone());
            }),
            missing_ui: Box::new(|ui: &mut Ui| {
                ui.label(missing_text);
            }),
        }
    }
}

pub struct OptionDisplayBuilder<'a, T>{
    option: &'a Option<T>,
    present_ui: Box<dyn FnOnce(&mut Ui, &T) + 'a>,
    missing_ui: Box<dyn FnOnce(&mut Ui) + 'a>,
}

impl<T> OptionDisplayBuilder<'_, T> {
    pub fn show(self, ui: &mut Ui) {
        match self.option {
            Some(val) => (self.present_ui)(ui, val),
            None => (self.missing_ui)(ui),
        }
    }
}
