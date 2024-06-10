#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(dead_code)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(extract_if)]
#![feature(iter_array_chunks)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::module_name_repetitions)]

mod apps;
mod db;
mod model;
mod utils;

use eframe::NativeOptions;
use egui::ViewportBuilder;

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: ViewportBuilder::default().with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::<apps::BreadApp>::default()),
    )
}
