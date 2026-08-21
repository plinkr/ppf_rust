mod app;
mod views;
mod worker;

use app::PpfApp;
use eframe::egui;

fn load_app_icon() -> Option<egui::IconData> {
    let icon_bytes = include_bytes!("../assets/app_logo.png");
    let image = image::load_from_memory(icon_bytes).ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    Some(egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    })
}

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([740.0, 640.0])
        .with_min_inner_size([680.0, 560.0])
        .with_title("PPF Rust Patcher")
        .with_drag_and_drop(true);

    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "PPF Rust Patcher",
        native_options,
        Box::new(|cc| Ok(Box::new(PpfApp::new(cc)))),
    )
}
