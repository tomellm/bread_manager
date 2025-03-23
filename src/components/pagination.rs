use egui::Ui;

#[derive(Clone)]
pub struct PaginationControls {
    pub page: usize,
    pub per_page: usize,
}

impl PaginationControls {
    pub fn controls(&mut self, ui: &mut Ui, num_elements: usize) {
        ui.horizontal(|ui| {
            if ui.button("<").clicked() && self.page > 0 {
                self.page -= 1;
            }
            if ui.button("-10").clicked() && self.per_page > 10 {
                self.per_page -= 10;
            }
            if ui.button("-").clicked() && self.per_page > 1 {
                self.per_page -= 1;
            }
            if ui.button("+").clicked() {
                self.per_page += 1;
            }
            if ui.button("+10").clicked() {
                self.per_page += 10;
            }
            if ui.button(">").clicked()
                && self.page < num_elements / self.per_page
            {
                self.page += 1;
            }
        });
    }

    pub fn page_info(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(format!("Page: {}", self.page + 1));
            ui.label(format!("Per Page: {}", self.per_page));
        });
    }
}

impl Default for PaginationControls {
    fn default() -> Self {
        Self {
            page: 0,
            per_page: 30,
        }
    }
}

pub trait Paginator<T> {
    fn paginate<'a>(
        &'a self,
        controls: &'a PaginationControls,
    ) -> Option<impl IntoIterator<Item = &'a T>>
    where
        T: 'a;
}

impl<T> Paginator<T> for Vec<T> {
    fn paginate<'a>(
        &'a self,
        controls: &'a PaginationControls,
    ) -> Option<impl IntoIterator<Item = &'a T>>
    where
        T: 'a,
    {
        self.chunks(controls.per_page).nth(controls.page)
    }
}

impl<T> Paginator<T> for &[T] {
    fn paginate<'a>(
        &'a self,
        controls: &PaginationControls,
    ) -> Option<impl IntoIterator<Item = &'a T>>
    where
        T: 'a,
    {
        self.chunks(controls.per_page).nth(controls.page)
    }
}
