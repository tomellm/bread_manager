use core::f64;
use std::{collections::HashMap, f64::consts::E};

use data_communicator::buffered::{
    change::ChangeResult, communicator::Communicator, query::QueryType, GetKeys,
};
use eframe::App;
use egui::{
    CentralPanel, Color32, Context, Frame, Grid, Label, Response, RichText, ScrollArea, Sense,
    SidePanel, Ui, UiBuilder, Widget,
};
use egui_light_states::{future_await::FutureAwait, UiStates};
use lazy_async_promise::ImmediateValuePromise;
use uuid::Uuid;

use crate::{
    components::{expense_record::RecordListView, option_display::OptionDisplay},
    model::{
        linker::{Link, PossibleLink},
        records::{ExpenseRecord, ExpenseRecordUuid},
    },
};

use super::utils::{drag_int, drag_zero_to_one};

pub struct Linking {
    records: Communicator<Uuid, ExpenseRecord>,
    links: Communicator<Uuid, Link>,
    possible_links: Communicator<Uuid, PossibleLink>,
    state: UiStates,
    falloff_steepness: f64,
    offset_days: usize,
    anchor: LinksAnchor,
}

impl App for Linking {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.possible_links.state_update();
        self.records.state_update();
        self.links.state_update();
        self.records.state_update();

        CentralPanel::default().show(ctx, |ui| {
            SidePanel::left("possible_link_scroll_area")
                .min_width(300.)
                .resizable(true)
                .show_inside(ui, |ui| {
                    ui.heading("Click which one to view");
                    ui.horizontal(|ui| {
                        ui.set_width(300.);
                        if ui.button("Possible Links").clicked() {
                            self.anchor.switch_possible_links();
                        }
                        if ui.button("Links").clicked() {
                            self.anchor.switch_links();
                        }
                    });
                    ui.heading("Probability Recalculation");
                    ui.horizontal(|ui| {
                        drag_int(ui, &mut self.offset_days);
                        drag_zero_to_one(ui, &mut self.falloff_steepness);
                        if ui.button("recalk probability").clicked() {
                            let promise = self.calculate_probability();
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
                    ui.label(format!(
                        "Currently {} possible links",
                        self.possible_links.len()
                    ));
                    ScrollArea::both().show(ui, |ui| match self.anchor {
                        LinksAnchor::PossibleLinks(_) => self.list_possible_links(ui),
                        LinksAnchor::Links(_) => self.list_links(ui),
                    });
                });
            CentralPanel::default().show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("delete all possible links").clicked() {
                        let delete_future = self
                            .possible_links
                            .delete_many(self.possible_links.data_map().keys().cloned().collect());
                        self.state
                            .set_future("delete_all_possible_links")
                            .set(delete_future);
                    }
                    self.state
                        .future_status::<ChangeResult>("delete_all_possible_links")
                        .default()
                        .show(ui);

                    if ui.button("delete all links").clicked() {
                        let delete_future = self
                            .links
                            .delete_many(self.links.data_map().keys().cloned().collect());
                        self.state.set_future("delete_all_links").set(delete_future);
                    }
                    self.state
                        .future_status::<ChangeResult>("delete_all_links")
                        .default()
                        .show(ui);
                });

                ui.separator();
                self.view_selected_link(ui);
            });
        });
    }
}

impl Linking {
    pub fn new(
        records: Communicator<Uuid, ExpenseRecord>,
        links: Communicator<Uuid, Link>,
        mut possible_links: Communicator<Uuid, PossibleLink>,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            let _ = records.query_future(QueryType::All).await;
            let _ = links.query_future(QueryType::All).await;
            let _ = possible_links.query_future(QueryType::All).await;
            possible_links.sort(|a, b| b.probability.total_cmp(&a.probability));
            Self {
                records,
                links,
                possible_links,
                state: UiStates::default(),
                falloff_steepness: 0.,
                offset_days: 5,
                anchor: LinksAnchor::default(),
            }
        }
    }

    fn list_possible_links(&mut self, ui: &mut Ui) {
        for possible_link in self.possible_links.data_sorted() {
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
                self.anchor.set_possible_link(possible_link.clone());
            }
        }
    }

    fn list_links(&mut self, ui: &mut Ui) {
        for link in self.links.data_iter() {
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
                self.anchor.set_link(link.clone());
            }
        }
    }

    fn view_selected_link(&mut self, ui: &mut Ui) -> Response {
        ui.group(|ui| match &self.anchor.clone() {
            LinksAnchor::PossibleLinks(Some(link)) => {
                self.view_possible_link(ui, link);
                self.view_records(ui, &link.negative, &link.positive);
            }
            LinksAnchor::Links(Some(link)) => {
                self.view_link(ui, link);
                self.view_records(ui, &link.negative, &link.positive);
            }
            _ => {
                ui.label("click on a uuid in the list to view details");
            }
        })
        .response
    }

    fn view_possible_link(&mut self, ui: &mut Ui, link: &PossibleLink) {
        Grid::new("selected_link_view").show(ui, |ui| {
            ui.label("Link Uuid:");
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
                if ui.button("save").clicked() {
                    self.create_link(link.clone());
                }
                self.state
                    .future_status::<()>("save_possible_link")
                    .default()
                    .show(ui);
            });
            ui.end_row();
        });
    }

    fn view_link(&mut self, ui: &mut Ui, link: &Link) {
        Grid::new("selected_link_view").show(ui, |ui| {
            ui.label("Link Uuid:");
            ui.label(format!("{}", link.uuid));
            ui.end_row();

            ui.label("Negative Side Uuid:");
            ui.label(format!("{}", *link.negative));
            ui.end_row();

            ui.label("Positive Side Uuid:");
            ui.label(format!("{}", *link.positive));
            ui.end_row();
        });
    }

    fn view_records(
        &self,
        ui: &mut Ui,
        negative: &ExpenseRecordUuid,
        positive: &ExpenseRecordUuid,
    ) {
        ui.horizontal(|ui| {
            ui.vertical_centered(|ui| {
                ui.group(|ui| {
                    self.view_record(ui, negative);
                });
            });
            ui.vertical_centered(|ui| {
                ui.group(|ui| {
                    self.view_record(ui, positive);
                });
            });
        });
    }

    fn view_record(&self, ui: &mut Ui, record: &ExpenseRecordUuid) {
        self.records
            .data_map()
            .get(&**record)
            .display(
                |ui, val| {
                    ui.add(RecordListView::new(val));
                },
                |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label("... Error, this record could not be found ...");
                        ui.label(format!("Record Uuid is: {}", **record));
                    });
                },
            )
            .show(ui);
    }

    fn create_link(&mut self, link: PossibleLink) {
        let mut links_to_delete = self.possible_links.data_iter().filter_map(|all_links| {
            if all_links.contains(&link.negative) || all_links.contains(&link.positive) {
                Some(link.uuid)
            } else { None }
        }).collect::<Vec<_>>();
        links_to_delete.push(link.uuid);

        let delete_future = self.possible_links.delete_many_future(links_to_delete);
        let create_future = self.links.insert_future(link.clone().into());
        self.state.set_future("save_possible_link").set(async move {
            let _ = create_future.await;
            let _ = delete_future.await;
        });
    }

    fn calculate_probability(&mut self) -> ImmediateValuePromise<()> {
        let falloff_steepness = self.falloff_steepness;
        let offset_days = self.offset_days as f64;

        let linked_records = self
            .possible_links
            .data_iter()
            .flat_map(|link| vec![*link.positive, *link.negative])
            .collect::<Vec<_>>();
        let records = self
            .records
            .data_map()
            .iter()
            .filter_map(|(key, val)| {
                if linked_records.contains(key) {
                    Some((*key, val.clone()))
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>();
        let links = self.possible_links.data_cloned();
        let mut update_many = self.possible_links.update_many_action();
        async move {
            let probs = links
                .iter()
                .map(|link| {
                    let Some(positive) = records.get(&link.positive) else {
                        return (link.uuid, f64::INFINITY);
                    };
                    let Some(negative) = records.get(&link.negative) else {
                        return (link.uuid, f64::INFINITY);
                    };

                    let time_distance = (*positive.datetime() - *negative.datetime())
                        .num_days()
                        .abs() as f64;
                    (link.uuid, time_distance)
                })
                .collect::<HashMap<_, _>>();

            let links = links
                .into_iter()
                .map(|mut link| {
                    let time_distance = probs.get(&link.uuid).unwrap();
                    link.probability = 1f64
                        / (1f64 + E.powf((1f64 - falloff_steepness) * time_distance - offset_days));
                    link
                })
                .collect::<Vec<_>>();

            let _ = update_many(links).await;
        }
        .into()
    }
}

#[derive(Clone)]
enum LinksAnchor {
    PossibleLinks(Option<PossibleLink>),
    Links(Option<Link>),
}

impl LinksAnchor {
    fn set_possible_link(&mut self, link: PossibleLink) {
        *self = Self::PossibleLinks(Some(link));
    }
    fn set_link(&mut self, link: Link) {
        *self = Self::Links(Some(link));
    }
    fn switch_possible_links(&mut self) {
        *self = Self::PossibleLinks(None);
    }
    fn switch_links(&mut self) {
        *self = Self::Links(None);
    }
}

impl Default for LinksAnchor {
    fn default() -> Self {
        Self::PossibleLinks(None)
    }
}
