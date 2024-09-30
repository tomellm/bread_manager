mod fileupload;
mod profiles;
mod tableview;
mod utils;
mod visualizations;

use std::time::{Duration, Instant};

use eframe::egui;
use tokio::sync::mpsc;

use crate::db::DB;

use self::{
    fileupload::FileUpload, profiles::Profiles, tableview::TableView,
    visualizations::Visualizations,
};

pub struct State {
    db: DB,
    file_upload: FileUpload,
    table_view: TableView,
    profiles: Profiles,
    visualizations: Visualizations,
    selected_anchor: Anchor,
}

pub struct BreadApp {
    state: State,
    send_dropped_file_upload: mpsc::Sender<egui::DroppedFile>,
    send_dropped_profiles: mpsc::Sender<egui::DroppedFile>,
    update_callback_ctx: Option<egui::Context>,
    render_start: Instant,
    frames_passed: usize,
    fps: f32,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum Anchor {
    Visualizations,
    FileUpload,
    TableView,
    Profiles,
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
                update_callback_ctx: None,
                render_start: Instant::now(),
                frames_passed: 0,
                fps: 0f32
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
        ];

        vec.into_iter()
    }

    fn bar_contents(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::widgets::global_dark_light_mode_switch(ui);

        ui.separator();

        self.frames_passed += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.render_start);
        if elapsed.as_secs() >= 1 {
            self.render_start = now;
            self.fps = self.frames_passed as f32 / elapsed.as_secs_f32();
            self.frames_passed = 0;
        }

        ui.label(format!("{:.0}", self.fps));

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
                println!("{:?}", self.state.selected_anchor);
                if let Some(sender) = self.get_current_sender() {
                    println!("{sender:?}");
                    tokio::spawn(async move {
                        for file in files {
                            let _ = sender.send(file).await;
                            callback();
                        }
                    });
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
}

impl eframe::App for BreadApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.state.db.state_update();
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

            let file_upload = FileUpload::new(rx_f, db.profiles_signal(), db.records_signal());
            Self {
                file_upload,
                profiles: Profiles::new(rx_p, [db.profiles_signal(), db.profiles_signal()]),
                table_view: TableView::new(db.records_signal()),
                visualizations: Visualizations::new(db.records_signal()),
                selected_anchor: Anchor::Visualizations,
                db,
            }
        }
    }
}
