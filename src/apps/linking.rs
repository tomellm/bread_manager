mod links;
mod possible_links;

use eframe::App;
use egui::{CentralPanel, Context, ScrollArea, SidePanel, TopBottomPanel, Ui};
use hermes::factory::Factory;
use links::LinksView;
use possible_links::PossibleLinksView;

use crate::{
    components::{expense_records::list_view::RecordListView, option_display::OptionDisplay},
    model::records::ExpenseRecord,
};

pub struct Linking {
    possible_links: PossibleLinksView,
    links: LinksView,
    anchor: Anchor,
}

impl App for Linking {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.possible_links.state_update();
        self.links.state_update();

        CentralPanel::default().show(ctx, |ui| {
            ui.label(ui.available_width().to_string());
            TopBottomPanel::top("possible_links_top_panel").show_inside(ui, |ui| {
                ui.heading("Click which one to view");
                ui.horizontal(|ui| {
                    ui.set_width(300.);
                    if ui.button("Possible Links").clicked() {
                        self.anchor = Anchor::PossibleLinks;
                    }
                    if ui.button("Links").clicked() {
                        self.anchor = Anchor::Links;
                    }
                });
            });
            SidePanel::left("possible_link_scroll_area")
                .min_width(300.)
                .resizable(true)
                .show_inside(ui, |ui| {
                    ScrollArea::both().show(ui, |ui| match self.anchor {
                        Anchor::PossibleLinks => self.possible_links.list(ui),
                        Anchor::Links => self.links.list(ui),
                    });
                });
            CentralPanel::default().show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    self.possible_links.delete_all(ui);
                    self.links.delete_all(ui);
                });

                ui.separator();
                match self.anchor {
                    Anchor::PossibleLinks => self.possible_links.view_link(ui),
                    Anchor::Links => self.links.view_link(ui),
                }
            });
        });
    }
}

impl Linking {
    pub fn init(factory: Factory) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            Self {
                possible_links: PossibleLinksView::init(factory.clone()).await,
                links: LinksView::init(factory).await,
                anchor: Anchor::default(),
            }
        }
    }
}

#[derive(Clone, Default)]
enum Anchor {
    #[default]
    PossibleLinks,
    Links,
}

fn view_records(negative: Option<&ExpenseRecord>, positive: Option<&ExpenseRecord>, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.group(|ui| {
            ui.set_width(ui.available_width() * 0.5);
            ui.set_height(ui.available_height());
            ui.vertical(|ui| {
                ui.heading("Negative Side:");
                view_record(negative, ui);
            });
        });
        ui.group(|ui| {
            ui.set_width(ui.available_width() * 0.5);
            ui.set_height(ui.available_height());
            ui.vertical(|ui| {
                ui.heading("Positive Side:");
                view_record(positive, ui);
            });
        });
    });
}

fn view_record(record: Option<&ExpenseRecord>, ui: &mut Ui) {
    record
        .display(
            |ui, val| {
                ui.add(RecordListView::new(val));
            },
            |ui| {
                ui.vertical_centered(|ui| {
                    ui.label("... Error, this record could not be found ...");
                });
            },
        )
        .show(ui);
}
