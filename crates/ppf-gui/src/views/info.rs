use crate::app::PpfApp;
use crate::worker::{FilePickTarget, InspectTargetTab, pick_file_async};
use egui::{Color32, RichText, Ui};
use ppf_core::{ImageType, PpfVersion};

pub fn show(app: &mut PpfApp, ui: &mut Ui) {
    ui.add_space(8.0);

    // Card 1: File Selection
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.heading("Inspect Patch File");
        ui.add_space(4.0);

        ui.label(RichText::new("Select PPF Patch to inspect (.ppf):").strong());
        ui.horizontal(|ui| {
            let edit = ui.add(
                egui::TextEdit::singleline(&mut app.info_patch_path)
                    .hint_text("/path/to/patch.ppf or drag & drop file here")
                    .desired_width(ui.available_width() - 100.0),
            ).on_hover_text(
                "Path to the PPF patch file (PPF 1.0, 2.0, or 3.0).\nYou can also drag & drop the file directly into the window.",
            );
            if edit.changed() {
                app.info_patch_info = None;
                app.info_inspect_error = None;
                app.inspect_patch_if_exists(InspectTargetTab::Info);
            }

            let browse_btn = ui.add_enabled(!app.is_busy, egui::Button::new("Browse..."))
                .on_hover_text("Open file chooser to select PPF patch file");
            if browse_btn.clicked() {
                pick_file_async(
                    FilePickTarget::InfoPatch,
                    "Select PPF Patch File to Inspect",
                    vec![("PPF Patches", &["ppf"]), ("All Files", &["*"])],
                    app.tx_event.clone(),
                );
            }
        });
    });

    ui.add_space(10.0);

    // Card 2: Patch Metadata
    if let Some(info) = &app.info_patch_info {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.heading("Patch Header Information");
            ui.add_space(6.0);

            egui::Grid::new("info_patch_details_grid")
                .num_columns(2)
                .spacing([20.0, 8.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Format Version:").strong())
                        .on_hover_text("PPF specification version used to create this patch.");
                    let ver_str = match info.version {
                        PpfVersion::V1 => "PPF 1.0 (Legacy format)",
                        PpfVersion::V2 => "PPF 2.0 (Blockcheck format)",
                        PpfVersion::V3 => "PPF 3.0 (Advanced format)",
                    };
                    ui.label(
                        RichText::new(ver_str)
                            .strong()
                            .color(Color32::from_rgb(90, 180, 255)),
                    );
                    ui.end_row();

                    ui.label(RichText::new("Description:").strong())
                        .on_hover_text("Title, credit, or description embedded in the patch header.");
                    ui.label(RichText::new(&info.description).italics());
                    ui.end_row();

                    if info.version == PpfVersion::V3 {
                        ui.label(RichText::new("Image Type:").strong())
                            .on_hover_text("Expected disc image format (BIN RAW vs GI Global Image).");
                        let img_type_str = match info.image_type {
                            ImageType::Bin => "BIN (Standard RAW / ISO)",
                            ImageType::Gi => "GI (Global Image / PrimoDVD)",
                        };
                        ui.label(img_type_str);
                        ui.end_row();

                        ui.label(RichText::new("Block Validation:").strong())
                            .on_hover_text("1024-byte checksum verification protecting against incorrect source files.");
                        if info.block_check {
                            ui.label(
                                RichText::new("Enabled (1024-byte checksum verification)")
                                    .color(Color32::from_rgb(100, 220, 120)),
                            );
                        } else {
                            ui.label(RichText::new("Disabled").color(Color32::from_rgb(200, 200, 200)));
                        }
                        ui.end_row();

                        ui.label(RichText::new("Undo Data:").strong())
                            .on_hover_text("Original byte restoration data allowing reversing changes.");
                        if info.has_undo {
                            ui.label(
                                RichText::new("Available (Patch can be reversed)")
                                    .color(Color32::from_rgb(100, 220, 120))
                                    .strong(),
                            );
                        } else {
                            ui.label(
                                RichText::new("Not available (One-way patch)")
                                    .color(Color32::from_rgb(220, 160, 80)),
                            );
                        }
                        ui.end_row();
                    }

                    ui.label(RichText::new("FILE_ID.DIZ:").strong())
                        .on_hover_text("Presence of release description / BBS info text.");
                    if info.file_id.is_some() {
                        ui.label(
                            RichText::new("Embedded in patch trailer")
                                .color(Color32::from_rgb(100, 220, 120)),
                        );
                    } else {
                        ui.label(RichText::new("None").color(Color32::from_rgb(160, 160, 160)));
                    }
                    ui.end_row();
                });
        });

        // Card 3: FILE_ID.DIZ Viewer (if present)
        if let Some(diz) = &info.file_id {
            ui.add_space(10.0);
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.heading("FILE_ID.DIZ Viewer");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let copy_btn = ui.button("Copy Text").on_hover_text(
                            "Copy the entire FILE_ID.DIZ text to the system clipboard.",
                        );
                        if copy_btn.clicked() {
                            ui.output_mut(|o| o.copied_text = diz.clone());
                        }
                    });
                });
                ui.add_space(6.0);

                egui::ScrollArea::vertical()
                    .max_height(240.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::Label::new(
                                RichText::new(diz)
                                    .monospace()
                                    .color(Color32::from_rgb(200, 230, 255)),
                            )
                            .wrap(),
                        );
                    });
            });
        }
    } else if let Some(err) = &app.info_inspect_error {
        ui.add_space(10.0);
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.colored_label(
                Color32::from_rgb(255, 100, 100),
                format!("Failed to parse PPF patch: {}", err),
            );
        });
    } else {
        ui.add_space(20.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("Select or drag & drop a PPF patch file above to inspect its metadata and embedded release information.")
                    .color(Color32::from_rgb(160, 160, 160))
                    .size(14.0),
            );
        });
    }
}
