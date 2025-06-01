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
pub mod infra;
mod model;
mod utils;

use apps::BreadApp;
use eframe::NativeOptions;
use egui::ViewportBuilder;
use infra::sqlx_layer::SqlxLayer;
use tracing_subscriber::{
    filter::filter_fn, fmt, prelude::*, EnvFilter, Registry,
};
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

    //let stdout_log = tracing_subscriber::fmt::layer();
    //tracing_subscriber::registry()
    //
    //    .init();

    let sqlx_layer = SqlxLayer;

    let fmt_subscriber = fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_filter(EnvFilter::from_env("LOG_FILTER"))
        // remove original sqlx::query events from log output
        .with_filter(filter_fn(|metadata| metadata.target() != "sqlx::query"));

    Registry::default()
        .with(fmt_subscriber)
        .with(sqlx_layer)
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
