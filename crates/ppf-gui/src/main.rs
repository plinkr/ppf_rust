mod app;
mod views;
mod worker;

use app::PpfApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([740.0, 580.0])
            .with_min_inner_size([640.0, 520.0])
            .with_title("PPF Rust Patcher")
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "PPF Rust Patcher",
        native_options,
        Box::new(|cc| Ok(Box::new(PpfApp::new(cc)))),
    )
}
