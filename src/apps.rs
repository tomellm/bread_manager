mod fileupload;
mod linking;
mod profiles;
mod tableview;
pub(crate) mod utils;
mod visualizations;

use std::time::Instant;

use data_communicator::buffered::utils::PromiseUtilities;
use eframe::{egui, App};
use egui::global_theme_preference_switch;
use lazy_async_promise::{ImmediateValuePromise, ImmediateValueState};
use linking::Linking;
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::{db::DB, utils::LoadingScreen};

use self::{
    fileupload::FileUpload, profiles::Profiles, tableview::TableView,
    visualizations::Visualizations,
};

pub struct Fps {
    render_start: Instant,
    frames_passed: usize,
    fps: f32,
}

pub struct State {
    db: DB,
    file_upload: LoadingScreen<FileUpload>,
    table_view: LoadingScreen<TableView>,
    profiles: LoadingScreen<Profiles>,
    visualizations: LoadingScreen<Visualizations>,
    linking: LoadingScreen<Linking>,
    selected_anchor: Anchor,
    fps: Fps,
}

pub struct BreadApp {
    state: State,
    send_dropped_file_upload: mpsc::Sender<egui::DroppedFile>,
    send_dropped_profiles: mpsc::Sender<egui::DroppedFile>,
    sending_files: Vec<ImmediateValuePromise<()>>,
    update_callback_ctx: Option<egui::Context>,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum Anchor {
    Visualizations,
    FileUpload,
    TableView,
    Profiles,
    Linking,
}

impl BreadApp {
    pub fn init() -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            let (tx_f, rx_f) = mpsc::channel::<egui::DroppedFile>(20);
            let (tx_p, rx_p) = mpsc::channel::<egui::DroppedFile>(20);

            Self {
                state: State::init(rx_f, rx_p).await,
                send_dropped_file_upload: tx_f,
                send_dropped_profiles: tx_p,
                sending_files: vec![],
                update_callback_ctx: None,
            }
        }
    }

    fn apps_iter_mut(&mut self) -> impl Iterator<Item = (&str, Anchor, &mut dyn eframe::App)> {
        let vec = vec![
            (
                "Visualizations",
                Anchor::Visualizations,
                &mut self.state.visualizations as &mut dyn eframe::App,
            ),
            (
                "Table View",
                Anchor::TableView,
                &mut self.state.table_view as &mut dyn eframe::App,
            ),
            (
                "File Upload",
                Anchor::FileUpload,
                &mut self.state.file_upload as &mut dyn eframe::App,
            ),
            (
                "Profiles",
                Anchor::Profiles,
                &mut self.state.profiles as &mut dyn eframe::App,
            ),
            (
                "Linking",
                Anchor::Linking,
                &mut self.state.linking as &mut dyn eframe::App,
            ),
        ];

        vec.into_iter()
    }

    fn bar_contents(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        global_theme_preference_switch(ui);

        ui.separator();

        ui.label(format!("{:.0}", self.state.fps.fps));

        ui.separator();

        let mut selected_anchor = self.state.selected_anchor;
        for (name, anchor, _) in self.apps_iter_mut() {
            if ui
                .selectable_label(selected_anchor == anchor, name)
                .clicked()
            {
                selected_anchor = anchor;
            }
        }
        self.state.selected_anchor = selected_anchor;
    }

    fn show_selected_app(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let selected_anchor = self.state.selected_anchor;
        for (_, anchor, app) in self.apps_iter_mut() {
            if anchor == selected_anchor || ctx.memory(egui::Memory::everything_is_visible) {
                app.update(ctx, frame);
            }
        }
    }

    fn ui_file_drag_and_drop(&mut self, ctx: &egui::Context) {
        use egui::{Align2, Color32, Id, LayerId, Order, TextStyle};
        use std::fmt::Write as _;

        if ![Anchor::FileUpload, Anchor::Profiles].contains(&self.state.selected_anchor) {
            return;
        }

        // Preview hovering files:
        if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
            let text = ctx.input(|i| {
                let mut text = "Dropping files:\n".to_owned();
                for file in &i.raw.hovered_files {
                    if let Some(path) = &file.path {
                        write!(text, "\n{}", path.display()).ok();
                    } else if !file.mime.is_empty() {
                        write!(text, "\n{}", file.mime).ok();
                    } else {
                        text += "\n???";
                    }
                }
                text
            });

            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

            let screen_rect = ctx.screen_rect();
            painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                screen_rect.center(),
                Align2::CENTER_CENTER,
                text,
                TextStyle::Heading.resolve(&ctx.style()),
                Color32::WHITE,
            );
        }

        // Collect dropped files:
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                let callback = self.update_callback();
                let files = i.raw.dropped_files.clone();
                info!(
                    msg = format!(
                        "Recived Dropped file, sending to App [{:?}]",
                        self.state.selected_anchor
                    )
                );
                if let Some(sender) = self.get_current_sender() {
                    self.sending_files.push(
                        async move {
                            for file in files {
                                let _ = sender.send(file).await;
                                callback();
                            }
                        }
                        .into(),
                    );
                }
            }
        });
    }

    pub fn update_callback(&self) -> impl Fn() {
        let ctx = self.update_callback_ctx.clone().unwrap();
        move || ctx.request_repaint()
    }

    pub fn get_current_sender(&self) -> Option<mpsc::Sender<egui::DroppedFile>> {
        match self.state.selected_anchor {
            Anchor::FileUpload => Some(self.send_dropped_file_upload.clone()),
            Anchor::Profiles => Some(self.send_dropped_profiles.clone()),
            _ => None,
        }
    }

    pub fn update_sending_files(&mut self) {
        self.sending_files
            .extract_if(PromiseUtilities::poll_and_check_finished)
            .for_each(|mut result| {
                if let ImmediateValueState::Error(err) = result.poll_state() {
                    warn!(
                        msg = format!("Sending a dropped file resulted in an error: [{}]", err.0)
                    );
                };
            })
    }
}

impl App for BreadApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.state.db.state_update();
        self.state.fps.update_fps();
        self.update_callback_ctx = Some(ctx.clone());

        egui::TopBottomPanel::top("wrap_app_top_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.visuals_mut().button_frame = false;
                self.bar_contents(ui, frame);
            });
        });
        self.show_selected_app(ctx, frame);
        self.ui_file_drag_and_drop(ctx);
    }
}

impl State {
    fn init(
        rx_f: mpsc::Receiver<egui::DroppedFile>,
        rx_p: mpsc::Receiver<egui::DroppedFile>,
    ) -> impl std::future::Future<Output = Self> + Send + 'static {
        async move {
            let mut db = DB::get_db(false).await.unwrap();

            Self {
                file_upload: FileUpload::init(
                    rx_f,
                    db.profiles_signal(),
                    db.records_signal(),
                    db.possible_links_signal(),
                )
                .into(),
                profiles: Profiles::init(rx_p, [db.profiles_signal(), db.profiles_signal()]).into(),
                table_view: TableView::init(db.records_signal()).into(),
                visualizations: Visualizations::init(db.records_signal()).into(),
                linking: Linking::new(db.records_signal(), db.possible_links_signal()).into(),
                selected_anchor: Anchor::Visualizations,
                db,
                fps: Fps::default(),
            }
        }
    }
}

impl Fps {
    fn update_fps(&mut self) {
        self.frames_passed += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.render_start);
        if elapsed.as_secs() >= 1 {
            self.render_start = now;
            self.fps = self.frames_passed as f32 / elapsed.as_secs_f32();
            self.frames_passed = 0;
        }
    }
}

impl Default for Fps {
    fn default() -> Self {
        Self {
            render_start: Instant::now(),
            frames_passed: 0,
            fps: 0.,
        }
    }
}
