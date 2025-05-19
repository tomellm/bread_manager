use egui::Ui;

pub trait OriginsDialog {
    fn origins_dialog(&mut self);
}

impl OriginsDialog for Ui {
    fn origins_dialog(&mut self) {
        Modal
    }
}
