#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod discord;
mod history;
mod i18n;
mod pack;
mod store;
mod theme;
mod ui;

use app::DisvokeApp;
use egui::{ColorImage, IconData, ViewportBuilder};
use std::sync::Arc;

const LOGO_PNG: &[u8] = include_bytes!("../assets/logo.png");

fn load_logo_assets() -> (Option<IconData>, Option<ColorImage>) {
    let Ok(img) = image::load_from_memory(LOGO_PNG) else {
        return (None, None);
    };

    let icon_rgba = img
        .resize_exact(64, 64, image::imageops::FilterType::Lanczos3)
        .to_rgba8();
    let icon = IconData {
        rgba: icon_rgba.as_raw().clone(),
        width: icon_rgba.width(),
        height: icon_rgba.height(),
    };

    let tb_rgba = img
        .resize_exact(36, 36, image::imageops::FilterType::Lanczos3)
        .to_rgba8();
    let tb_image = ColorImage::from_rgba_unmultiplied(
        [tb_rgba.width() as usize, tb_rgba.height() as usize],
        tb_rgba.as_raw(),
    );

    (Some(icon), Some(tb_image))
}

fn main() -> eframe::Result<()> {
    let rt = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to initialize tokio runtime"),
    );

    let (icon_data, logo_image) = load_logo_assets();
    let mut viewport = ViewportBuilder::default()
        .with_title("Disvoke")
        .with_inner_size([1100.0, 760.0])
        .with_min_inner_size([900.0, 600.0])
        .with_decorations(false);

    if let Some(icon) = icon_data {
        viewport = viewport.with_icon(Arc::new(icon));
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Disvoke",
        options,
        Box::new(move |cc| Ok(Box::new(DisvokeApp::new(cc, rt, logo_image)))),
    )
}
