use std::collections::HashMap;

use data_communicator::buffered::{
    change::ChangeResult, communicator::Communicator, query::QueryType,
};
use egui::{Color32, Frame, Grid, Label, RichText, Sense, Ui, UiBuilder, Widget};
use egui_light_states::{future_await::FutureAwait, UiStates};
use lazy_async_promise::ImmediateValuePromise;
use uuid::Uuid;

use crate::{
    apps::utils::{drag_int, drag_zero_to_one},
    components::{expense_records::list_view::RecordListView, pagination::PaginationControls},
    db::possible_links,
    model::{
        linker::{Link, Linker, PossibleLink},
        records::ExpenseRecord,
    },
};

pub(super) struct PossibleLinksView {
    possible_links: Communicator<Uuid, PossibleLink>,
    linker: Linker,
    pagination: PaginationControls,
    selected: Option<PossibleLink>,
    state: UiStates,
    falloff_steepness: f64,
    offset_days: usize,
}

impl PossibleLinksView {
    pub fn init(
        records: Communicator<Uuid, ExpenseRecord>,
        [mut possible_links_one, possible_links_two]: [Communicator<Uuid, PossibleLink>; 2],
        links: Communicator<Uuid, Link>,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            let _ = records.query(QueryType::All).await;
            let _ = possible_links_one.query(QueryType::All).await;
            possible_links_one.sort(|a, b| b.probability.total_cmp(&a.probability));
            Self {
                possible_links: possible_links_one,
                linker: Linker::init(possible_links_two, links, records).await,
                pagination: PaginationControls::default(),
                selected: None,
                state: UiStates::default(),
                falloff_steepness: 0.,
                offset_days: 5,
            }
        }
    }

    pub fn state_update(&mut self) {
        self.possible_links.state_update();
        self.linker.state_update();
    }

    pub(super) fn delete_all(&mut self, ui: &mut Ui) {
        if !self
            .state
            .is_running::<ChangeResult>("delete_all_possible_links")
            && ui.button("delete all possible links").clicked()
        {
            let delete_future = self
                .possible_links
                .delete_many(self.possible_links.data.keys_cloned());
            self.state
                .set_future("delete_all_possible_links")
                .set(delete_future);
        }
    }

    pub(super) fn list(&mut self, ui: &mut Ui) {
        if self.possible_links.is_empty() {
            ui.label(POSSIBLE_LINKS_EMPTY_TEXT);
            return;
        }

        ui.heading("Probability Recalculation");
        ui.horizontal(|ui| {
            drag_int(ui, &mut self.offset_days);
            drag_zero_to_one(ui, &mut self.falloff_steepness);
            if ui.button("recalk probability").clicked() {
                let promise = self
                    .linker
                    .calculate_probability(self.falloff_steepness, self.offset_days as f64);
                self.state
                    .set_future("recalculate_probability")
                    .set(promise);
            }
            self.state
                .future_status::<()>("recalculate_probability")
                .default()
                .show(ui);
        });
        ui.separator();

        self.pagination.controls(ui, self.possible_links.data.len());
        self.pagination.page_info(ui);
        for possible_link in self
            .possible_links
            .data
            .page(self.pagination.page, self.pagination.per_page)
            .unwrap()
        {
            let response = ui
                .scope_builder(
                    UiBuilder::new()
                        .id_salt(format!("possible_link_list_{}", possible_link.uuid))
                        .sense(Sense::click()),
                    |ui| {
                        ui.set_width(280.);
                        ui.set_height(25.);
                        let response = ui.response();
                        let visuals = ui.style().interact(&response);

                        let mut rect = ui.available_rect_before_wrap();
                        rect.set_right(
                            rect.right() - rect.width() * (1f32 - possible_link.probability as f32),
                        );
                        let frame = Frame::none()
                            .fill(Color32::from_rgba_unmultiplied(255, 0, 0, 20))
                            .stroke(visuals.bg_stroke)
                            .inner_margin(ui.spacing().menu_margin)
                            .paint(rect);
                        ui.painter().add(frame);

                        let response = ui.response();
                        let visuals = ui.style().interact(&response);
                        let text_color = visuals.text_color();

                        Frame::canvas(ui.style())
                            .fill(visuals.bg_fill.gamma_multiply(0.3))
                            .stroke(visuals.bg_stroke)
                            .inner_margin(ui.spacing().menu_margin)
                            .show(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    Label::new(
                                        RichText::new(format!("{}", possible_link.uuid))
                                            .color(text_color),
                                    )
                                    .selectable(false)
                                    .ui(ui);
                                });
                            });
                    },
                )
                .response;

            if response.clicked() {
                self.selected = Some(possible_link.clone());
            }
        }
    }

    pub(super) fn view_link(&mut self, ui: &mut Ui) {
        let Some(link) = &self.selected else {
            ui.vertical_centered(|ui| {
                ui.add_space(30.);
                ui.label(SELECTED_LINK_EMPTY_TEXT);
                ui.add_space(30.);
            });
            return;
        };

        Grid::new("selected_link_view").show(ui, |ui| {
            ui.heading("Link Uuid:");
            ui.label(format!("{}", link.uuid));
            ui.end_row();

            ui.label("Negative Side Uuid:");
            ui.label(format!("{}", *link.negative));
            ui.end_row();

            ui.label("Positive Side Uuid:");
            ui.label(format!("{}", *link.positive));
            ui.end_row();

            ui.label("Probability of beeing correct:");
            ui.label(format!("{:.2}%", link.probability * 100.));
            ui.end_row();

            ui.label("");
            ui.horizontal(|ui| {
                if !self.state.is_running::<()>("save_possible_link") && ui.button("save").clicked()
                {
                    let future = self.linker.create_link(link);
                    self.state.set_future("save_possible_link").set(future);
                }
            });
            ui.end_row();
        });
        let (negative, positive) = self.linker.get_records(link);
        super::view_records(negative, positive, ui);
    }
}

const POSSIBLE_LINKS_EMPTY_TEXT: &str = "There are currently no possible links";
const SELECTED_LINK_EMPTY_TEXT: &str = r#"
Click on any of the possible links in the list
to view details about it.
"#;
