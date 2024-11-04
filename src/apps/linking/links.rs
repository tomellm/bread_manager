use diesel::{QueryDsl, SelectableHelper};
use egui::{Frame, Grid, Label, RichText, Sense, Ui, UiBuilder, Widget};
use egui_light_states::UiStates;
use hermes::{container::projecting::ProjectingContainer, factory::Factory};

use crate::{
    apps::DbConn,
    components::pagination::PaginationControls,
    db::{
        link::{DbLink, LINK_FROM_DB_FN},
        records::{DbRecord, RECORDS_FROM_DB_FN},
    },
    model::{linker::Link, records::ExpenseRecord},
    schema::{
        expense_records::dsl::expense_records as records_table, links::dsl::links as links_table,
    },
};

pub(super) struct LinksView {
    links: ProjectingContainer<Link, DbLink, DbConn>,
    records: ProjectingContainer<ExpenseRecord, DbRecord, DbConn>,
    pagination: PaginationControls,
    selected: Option<Link>,
    state: UiStates,
}

impl LinksView {
    pub(super) fn init(
        factory: &Factory<DbConn>,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        let mut links = factory.builder().projector_arc(LINK_FROM_DB_FN.clone());
        let mut records = factory.builder().projector_arc(RECORDS_FROM_DB_FN.clone());

        async move {
            let _ = links.query(|| links_table.select(DbLink::as_select()));
            let _ = records.query(|| records_table.select(DbRecord::as_select()));

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
        if ui.button("delete all links").clicked() {
            self.links.execute(diesel::delete(links_table));
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
