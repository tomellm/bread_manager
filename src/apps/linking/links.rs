//use egui::{Frame, Grid, Label, RichText, Sense, Ui, UiBuilder, Widget};
//use egui_light_states::UiStates;
//use hermes::{
//    container::{data::ImplData, manual},
//    factory::Factory,
//};
//use log::info;
//
//use crate::{components::pagination::PaginationControls, model::transactions::Transaction};
//
//pub(super) struct LinksView {
//    links: manual::Container<Link>,
//    transacts: manual::Container<Transaction>,
//    pagination: PaginationControls,
//    selected: Option<Link>,
//    state: UiStates,
//}
//
//impl LinksView {
//    pub(super) fn init(
//        factory: Factory,
//    ) -> impl std::future::Future<Output = Self> + Send + 'static {
//        async move {
//            let links = factory.builder().name("links_view_links").manual();
//            let transacts = factory.builder().name("links_view_records").manual();
//
//            //links.stored_query(DbLink::find_all_active());
//            //records.stored_query(DbRecord::find().select());
//
//            Self {
//                links,
//                transacts,
//                pagination: PaginationControls::default(),
//                selected: None,
//                state: UiStates::default(),
//            }
//        }
//    }
//
//    pub fn state_update(&mut self) {
//        self.links.state_update(true);
//    }
//
//    pub(super) fn delete_all(&mut self, _ui: &mut Ui) {
//        //if ui.button("(hard) delete all links").clicked() {
//        //    self.links.execute(DbLink::delete_many());
//        //}
//    }
//
//    pub(super) fn list(&mut self, ui: &mut Ui) {
//        if self.links.data().is_empty() {
//            ui.label(LINKS_EMPTY_TEXT);
//            return;
//        }
//
//        self.pagination.controls(ui, self.links.data().len());
//        self.pagination.page_info(ui);
//        let selected_page = self
//            .links
//            .data()
//            .chunks(self.pagination.per_page)
//            .nth(self.pagination.page)
//            .unwrap();
//        for link in selected_page {
//            let response = ui
//                .scope_builder(
//                    UiBuilder::new()
//                        .id_salt(format!("possible_link_list_{}", link.uuid))
//                        .sense(Sense::click()),
//                    |ui| {
//                        ui.set_width(280.);
//                        ui.set_height(25.);
//
//                        let response = ui.response();
//                        let visuals = ui.style().interact(&response);
//                        let text_color = visuals.text_color();
//
//                        Frame::canvas(ui.style())
//                            .fill(visuals.bg_fill.gamma_multiply(0.3))
//                            .stroke(visuals.bg_stroke)
//                            .inner_margin(ui.spacing().menu_margin)
//                            .show(ui, |ui| {
//                                ui.vertical_centered(|ui| {
//                                    Label::new(
//                                        RichText::new(format!("{}", link.uuid))
//                                            .color(text_color),
//                                    )
//                                    .selectable(false)
//                                    .ui(ui);
//                                });
//                            });
//                    },
//                )
//                .response;
//
//            if response.clicked() {
//                self.selected = Some(link.clone());
//                info!("{:?}, {:?}", link.leading, link.following);
//            }
//        }
//    }
//
//    pub(super) fn view_link(&mut self, ui: &mut Ui) {
//        let Some(link) = &self.selected else {
//            ui.vertical_centered(|ui| {
//                ui.add_space(30.);
//                ui.label(SELECTED_LINK_EMPTY_TEXT);
//                ui.add_space(30.);
//            });
//            return;
//        };
//
//        Grid::new("selected_link_view").show(ui, |ui| {
//            ui.heading("Link Uuid:");
//            ui.label(format!("{}", link.uuid));
//            ui.end_row();
//
//            ui.label("Negative Side Uuid:");
//            ui.label(format!("{}", *link.leading));
//            ui.end_row();
//
//            ui.label("Positive Side Uuid:");
//            ui.label(format!("{}", *link.following));
//            ui.end_row();
//        });
//
//        let (negative, positive) = self.transacts.data().iter().fold(
//            (None, None),
//            |mut matches, record| {
//                if link.leading.eq(record.uuid()) {
//                    let _ = matches.0.insert(record);
//                }
//                if link.following.eq(record.uuid()) {
//                    let _ = matches.1.insert(record);
//                }
//                matches
//            },
//        );
//        super::view_records(negative, positive, ui);
//    }
//}
//
//const LINKS_EMPTY_TEXT: &str = "There are currently no links";
//const SELECTED_LINK_EMPTY_TEXT: &str = r#"
//Click on any of the links in the list
//to view details about it.
//"#;
