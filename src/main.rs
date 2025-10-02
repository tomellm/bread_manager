#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(iter_array_chunks)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::manual_async_fn)]
#![allow(dead_code)]
#![deny(clippy::unconditional_recursion)]

mod apps;
mod components;
mod db;
pub mod infra;
mod model;
mod utils;

use std::env::{self, current_dir};

use apps::BreadApp;
use eframe::NativeOptions;
use egui::ViewportBuilder;
use infra::sqlx_layer::SqlxLayer;
use itertools::Itertools;
use tracing::info;
use tracing_appender::{
    non_blocking,
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{
    filter::filter_fn,
    fmt::{self, format},
    prelude::*,
    EnvFilter, Registry,
};
use utils::LoadingScreen;

#[tokio::main]
async fn main() -> eframe::Result<()> {
    println!("starting bread_manager {}", chrono::Local::now());
    let args = env::args().collect_vec();

    load_env(&args);

    let _guard = setup_logging().await;

    info!("\n\n################ starting bread-manager ################\n\n");

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

fn load_env(args: &[String]) {
    const ENV_ARG: &str = "--env=";

    match args.iter().find(|opt| opt.starts_with(ENV_ARG)) {
        Some(arg) => {
            let prefix_len = ENV_ARG.len();
            let path = arg.chars().skip(prefix_len).collect::<String>();
            assert!(path.starts_with("/"));
            dotenv::from_path(&path).unwrap_or_else(|_| {
                panic!("could not find .env file in: {path}")
            });
        }
        None => {
            dotenv::dotenv().unwrap_or_else(|_| {
                panic!(
                    "could not find .env file in: {:?}",
                    current_dir().expect("could not determine current_dir")
                )
            });
        }
    };
}

async fn setup_logging() -> WorkerGuard {
    let (file_layer, guard) = {
        let logs_location = env::var("LOG_FILE_LOCATION").unwrap();
        let logs_filename = String::from("logs.log");

        let file_appender = RollingFileAppender::new(
            Rotation::NEVER,
            logs_location,
            logs_filename,
        );
        let (non_blocking_file, guard) = non_blocking(file_appender);

        let file_layer_format = format()
            .with_ansi(false)
            .with_target(true)
            .with_level(true)
            .json();

        (
            fmt::Layer::default()
                .event_format(file_layer_format)
                .with_writer(non_blocking_file),
            guard,
        )
    };

    let stdout_layer = {
        let stdout_layer_format =
            format().with_ansi(true).with_target(true).with_level(true);

        fmt::Layer::default()
            .event_format(stdout_layer_format)
            .with_writer(std::io::stdout)
            .with_filter(filter_fn(|metadata| {
                metadata.target() != "sqlx::query"
            }))
    };

    let sqlx_layer = SqlxLayer;

    let subscriber = Registry::default()
        .with(EnvFilter::from_env("LOG_FILTER"))
        .with(stdout_layer)
        .with(sqlx_layer)
        .with(file_layer);

    tracing::subscriber::set_global_default(subscriber)
        .expect("unable to set global subscriber");

    guard
}

#[cfg(test)]
mod tests {

    //#[test]
    //fn test() {
    //    assert_eq!(1, 0);
    //}
}
