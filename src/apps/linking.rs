use core::f64;
use std::{collections::HashMap, f64::consts::E};

use chrono::Duration;
use data_communicator::buffered::{communicator::Communicator, query::QueryType};
use eframe::App;
use egui::{
    CentralPanel, Color32, Context, Frame, Label, Response, RichText, ScrollArea, Sense, SidePanel,
    Ui, UiBuilder, Widget,
};
use egui_light_states::{default_promise_await::DefaultCreatePromiseAwait, UiStates};
use lazy_async_promise::ImmediateValuePromise;
use tracing::info;
use uuid::Uuid;

use crate::{
    components::{expense_record::RecordListView, option_display::OptionDisplay},
    model::{linker::PossibleLink, records::ExpenseRecord},
};

use super::utils::{drag_int, drag_zero_to_one};

pub struct Linking {
    records: Communicator<Uuid, ExpenseRecord>,
    possible_links: Communicator<Uuid, PossibleLink>,
    selected_link: Option<PossibleLink>,
    state: UiStates,
    falloff_steepness: f64,
    offset_days: usize,
}

impl App for Linking {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.possible_links.state_update();
        self.records.state_update();

        CentralPanel::default().show(ctx, |ui| {
            SidePanel::left("possible_link_scroll_area")
                .min_width(300.)
                .show_inside(ui, |ui| {
                    ui.label(format!(
                        "Currently {} possible links",
                        self.possible_links.len()
                    ));
                    ScrollArea::both().show(ui, |ui| {
                        self.list_possible_links(ui);
                    });
                });
            CentralPanel::default().show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    drag_int(ui, &mut self.offset_days);
                    ui.add(
                        egui::DragValue::new(&mut self.falloff_steepness)
                            .speed(0.001)
                            .max_decimals(4)
                            .range(-10f64..=1f64),
                    );
                    if ui.button("recalk probability").clicked() {
                        let promise = self.calculate_probability();
                        self.state
                            .default_promise_await("recalculate_probability".into())
                            .init_ui(|_, set_pomise| {
                                set_pomise(promise);
                            })
                            .show(ui);
                    }
                });
                ui.separator();
                if let Some(link) = &self.selected_link {
                    self.view_selected_link(ui, link)
                } else {
                    ui.label("No Link selected, click to select")
                }
            });
        });
    }
}

impl Linking {
    pub fn new(
        records: Communicator<Uuid, ExpenseRecord>,
        mut possible_links: Communicator<Uuid, PossibleLink>,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            let _ = records.query_future(QueryType::All).await;
            let _ = possible_links.query_future(QueryType::All).await;
            possible_links.sort(|a, b| b.probability.total_cmp(&a.probability));
            Self {
                records,
                possible_links,
                selected_link: None,
                state: UiStates::default(),
                falloff_steepness: 0.,
                offset_days: 5,
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
                self.selected_link = Some(possible_link.clone());
            }
        }
    }

    fn view_selected_link(&self, ui: &mut Ui, link: &PossibleLink) -> Response {
        ui.group(|ui| {
            ui.label(link.uuid.to_string());
            ui.label(link.positive.to_string());
            ui.label(link.negative.to_string());
            ui.label(link.probability.to_string());
            ui.horizontal(|ui| {
                ui.group(|ui| self.view_record(ui, &link.negative));
                ui.group(|ui| self.view_record(ui, &link.positive));
            });
        })
        .response
    }

    fn view_record(&self, ui: &mut Ui, record: &Uuid) {
        self.records
            .data_map()
            .get(record)
            .display(
                |ui, val| {
                    ui.add(RecordListView::new(val));
                },
                |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label("... Error, this record could not be found ...");
                        ui.label(format!("Record Uuid is: {}", record));
                    });
                },
            )
            .show(ui);
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

                    let time_distance =
                        (*positive.datetime() - *negative.datetime()).num_days().abs() as f64;
                    info!("distance {time_distance}");
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
