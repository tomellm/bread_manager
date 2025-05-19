#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(iter_array_chunks)]
#![feature(result_flattening)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::manual_async_fn)]
#![deny(clippy::unconditional_recursion)]

mod apps;
mod components;
mod db;
mod model;
mod utils;

use apps::BreadApp;
use eframe::NativeOptions;
use egui::ViewportBuilder;
use tracing_subscriber::{prelude::*, EnvFilter};
use utils::LoadingScreen;

#[tokio::main]
async fn main() -> eframe::Result<()> {
    //console_subscriber::init();
    let _ = dotenv::dotenv();

    //let log_file = OpenOptions::new()
    //    .truncate(true)
    //    .write(true)
    //    .create(true)
    //    .open("logs/logs.log")
    //    .unwrap();

    //let log = tracing_subscriber::fmt::layer()
    //    .event_format(json())
    //    .with_writer(Arc::new(log_file));

    let stdout_log = tracing_subscriber::fmt::layer();
    tracing_subscriber::registry()
        .with(
            stdout_log.with_filter(EnvFilter::from_env("LOG_FILTER")), //.and_then(log),
        )
        .init();

    let options = NativeOptions {
        viewport: ViewportBuilder::default().with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "Bread App",
        options,
        Box::new(|_cc| Ok(Box::new(LoadingScreen::from(BreadApp::init())))),
    )
}
