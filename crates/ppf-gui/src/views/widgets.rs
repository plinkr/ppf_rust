use crate::app::PpfApp;
use egui::{Color32, RichText, Ui};

/// Renders the progress bar card when the application is executing a background worker task.
pub fn render_progress(app: &PpfApp, ui: &mut Ui) {
    if app.is_busy {
        ui.add_space(10.0);
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(&app.busy_operation).strong());
            ui.add_space(2.0);
            ui.add(
                egui::ProgressBar::new(app.progress)
                    .show_percentage()
                    .animate(true),
            );
            ui.label(&app.progress_msg);
        });
    }
}

/// Renders a success or error banner for completed operations.
pub fn render_status_banner(status: Option<&Result<String, String>>, ui: &mut Ui) {
    if let Some(status) = status {
        ui.add_space(10.0);
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            match status {
                Ok(msg) => {
                    ui.label(
                        RichText::new(msg)
                            .color(Color32::from_rgb(100, 230, 120))
                            .size(14.0)
                            .strong(),
                    );
                }
                Err(err) => {
                    ui.label(
                        RichText::new(format!("Error: {}", err))
                            .color(Color32::from_rgb(255, 100, 100))
                            .size(14.0)
                            .strong(),
                    );
                }
            }
        });
    }
}
