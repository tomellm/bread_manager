use data_communicator::buffered::{
    change::ChangeResult, communicator::Communicator, query::QueryType,
};
use egui::{Frame, Grid, Label, RichText, Sense, Ui, UiBuilder, Widget};
use egui_light_states::{future_await::FutureAwait, UiStates};
use uuid::Uuid;

use crate::{components::pagination::PaginationControls, model::{linker::Link, records::ExpenseRecord}};

pub(super) struct LinksView {
    links: Communicator<Uuid, Link>,
    records: Communicator<Uuid, ExpenseRecord>,
    pagination: PaginationControls,
    selected: Option<Link>,
    state: UiStates,
}

impl LinksView {
    pub(super) fn init(
        links: Communicator<Uuid, Link>,
        records: Communicator<Uuid, ExpenseRecord>,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            let _ = links.query(QueryType::All).await;
            let _ = records.query(QueryType::All).await;
            Self {
                links,
                records,
                pagination: PaginationControls::default(),
                selected: None,
                state: UiStates::default(),
            }
        }
    }

    pub fn state_update(&mut self) {
        self.links.state_update();
    }

    pub(super) fn delete_all(&mut self, ui: &mut Ui) {
        if !self.state.is_running::<ChangeResult>("delete_all_links")
            && ui.button("delete all links").clicked()
        {
            let delete_future = self.links.delete_many(self.links.data.keys_cloned());
            self.state.set_future("delete_all_links").set(delete_future);
        }
    }

    pub(super) fn list(&mut self, ui: &mut Ui) {
        if self.links.is_empty() {
            ui.label(LINKS_EMPTY_TEXT);
            return;
        }

        self.pagination.controls(ui, self.links.data.len());
        self.pagination.page_info(ui);
        for link in self
            .links
            .data
            .page(self.pagination.page, self.pagination.per_page)
            .unwrap()
        {
            let response = ui
                .scope_builder(
                    UiBuilder::new()
                        .id_salt(format!("possible_link_list_{}", link.uuid))
                        .sense(Sense::click()),
                    |ui| {
                        ui.set_width(280.);
                        ui.set_height(25.);

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
                                        RichText::new(format!("{}", link.uuid)).color(text_color),
                                    )
                                    .selectable(false)
                                    .ui(ui);
                                });
                            });
                    },
                )
                .response;

            if response.clicked() {
                self.selected = Some(link.clone());
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
        });

        let (negative, positive) = (
            self.records.data.map().get(&link.negative),
            self.records.data.map().get(&link.positive),
        );
        super::view_records(negative, positive, ui);
    }
}

const LINKS_EMPTY_TEXT: &str = "There are currently no links";
const SELECTED_LINK_EMPTY_TEXT: &str = r#"
Click on any of the links in the list
to view details about it.
"#;
