use crate::app::{PpfApp, clean_path_str};
use crate::views::widgets;
use crate::worker::{FilePickTarget, create_patch_async, pick_file_async, save_file_async};
use egui::{RichText, Ui};
use ppf_core::{ImageType, PpfCreatorOptions};
use std::path::PathBuf;

pub fn show(app: &mut PpfApp, ui: &mut Ui) {
    ui.add_space(8.0);

    // Card 1: File Paths
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.heading("Source and Destination Files");
        ui.add_space(4.0);

        // Original Unpatched Binary
        ui.label(RichText::new("Original Binary File (Unmodified):").strong());
        ui.horizontal(|ui| {
            let edit = ui.add(
                egui::TextEdit::singleline(&mut app.create_original_path)
                    .hint_text("/path/to/original.bin or drag & drop file here")
                    .desired_width(ui.available_width() - 100.0),
            ).on_hover_text("Path to the clean, unmodified original binary or disc image.\nYou can also drag & drop the file directly into the window.");
            if edit.changed() {
                app.create_status = None;
            }

            let browse_orig = ui.add_enabled(!app.is_busy, egui::Button::new("Browse..."))
                .on_hover_text("Open file chooser to select original unpatched file");
            if browse_orig.clicked() {
                pick_file_async(
                    FilePickTarget::CreateOriginal,
                    "Select Original Unpatched Binary",
                    vec![
                        ("Disc Images", &["bin", "iso", "img", "cue", "raw"]),
                        ("All Files", &["*"]),
                    ],
                    app.tx_event.clone(),
                );
            }
        });

        ui.add_space(6.0);

        // Modified Binary
        ui.label(RichText::new("Modified Binary File (Patched/Translated):").strong());
        ui.horizontal(|ui| {
            let edit = ui.add(
                egui::TextEdit::singleline(&mut app.create_patched_path)
                    .hint_text("/path/to/modified.bin or drag & drop file here")
                    .desired_width(ui.available_width() - 100.0),
            ).on_hover_text("Path to the modified, translated, or hacked binary file.\nMust be identical in byte size to the original file.\nYou can also drag & drop the file directly into the window.");
            if edit.changed() {
                app.create_status = None;
            }

            let browse_mod = ui.add_enabled(!app.is_busy, egui::Button::new("Browse..."))
                .on_hover_text("Open file chooser to select modified binary image");
            if browse_mod.clicked() {
                pick_file_async(
                    FilePickTarget::CreatePatched,
                    "Select Modified Binary Image",
                    vec![
                        ("Disc Images", &["bin", "iso", "img", "cue", "raw"]),
                        ("All Files", &["*"]),
                    ],
                    app.tx_event.clone(),
                );
            }
        });

        ui.add_space(6.0);

        // Output PPF3 File
        ui.label(RichText::new("Output PPF3 Patch File (.ppf):").strong());
        ui.horizontal(|ui| {
            let edit = ui.add(
                egui::TextEdit::singleline(&mut app.create_output_path)
                    .hint_text("/path/to/output_patch.ppf")
                    .desired_width(ui.available_width() - 100.0),
            ).on_hover_text("Destination path for the newly generated .ppf patch file.");
            if edit.changed() {
                app.create_status = None;
            }

            let save_btn = ui.add_enabled(!app.is_busy, egui::Button::new("Save As..."))
                .on_hover_text("Choose destination folder and name for output .ppf file");
            if save_btn.clicked() {
                save_file_async(
                    FilePickTarget::CreateOutput,
                    "Save Output PPF3 Patch File",
                    "patch.ppf",
                    vec![("PPF Patch", &["ppf"])],
                    app.tx_event.clone(),
                );
            }
        });
    });

    ui.add_space(10.0);

    // Card 2: Options
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.heading("PPF3 Configuration & Metadata");
        ui.add_space(4.0);

        // Description
        ui.label(RichText::new("Patch Description (max 50 chars):").strong());
        ui.horizontal(|ui| {
            let edit = ui.add(
                egui::TextEdit::singleline(&mut app.create_description)
                    .hint_text(ppf_core::DEFAULT_DESCRIPTION)
                    .desired_width(ui.available_width() - 80.0),
            ).on_hover_text("Embedded text description (title, version, author credits) up to 50 characters.");
            if edit.changed() && app.create_description.len() > 50 {
                app.create_description.truncate(50);
            }
            ui.label(format!("{}/50", app.create_description.len()));
        });

        ui.add_space(6.0);

        // Image Type
        ui.horizontal(|ui| {
            ui.label(RichText::new("Image Type:").strong())
                .on_hover_text("Target disc image format offset for block validation checksum.");
            ui.radio_value(
                &mut app.create_image_type,
                ImageType::Bin,
                "BIN / RAW (Standard CD/DVD)",
            ).on_hover_text("Standard CD-ROM or DVD RAW/ISO image (validates 1024-byte checksum block at 0x9320).");
            ui.radio_value(
                &mut app.create_image_type,
                ImageType::Gi,
                "GI (Global Image / PrimoDVD)",
            ).on_hover_text("Global Image / PrimoDVD format (validates 1024-byte checksum block at 0x80A0).");
        });

        ui.add_space(6.0);

        // Checkboxes
        ui.checkbox(
            &mut app.create_block_check,
            RichText::new("Include Block Check (Validation)").strong(),
        ).on_hover_text(
            "Checked (Recommended): Embeds a 1024-byte checksum from your original file into the patch.\nWhen users apply this patch, it verifies their file matches your original exact dump.",
        );

        ui.add_space(4.0);

        ui.checkbox(
            &mut app.create_undo_data,
            RichText::new("Include Undo Data (Reversible Patch)").strong(),
        ).on_hover_text(
            "Checked: Stores both original and modified bytes in the patch, allowing users to reverse it later.\n(Increases patch file size slightly).",
        );

        ui.add_space(6.0);

        // Optional FILE_ID.DIZ
        ui.label(RichText::new("Optional FILE_ID.DIZ File:").strong());
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut app.create_file_id_path)
                    .hint_text("Optional FILE_ID.DIZ or .txt release info")
                    .desired_width(ui.available_width() - 170.0),
            ).on_hover_text("Path to an optional release note or info text file (up to 3KB) embedded in the patch trailer.");

            let browse_diz = ui.add_enabled(!app.is_busy, egui::Button::new("Browse..."))
                .on_hover_text("Select a FILE_ID.DIZ, .txt, or .nfo file to embed");
            if browse_diz.clicked() {
                pick_file_async(
                    FilePickTarget::CreateFileId,
                    "Select FILE_ID.DIZ File",
                    vec![
                        ("DIZ and Text Files", &["diz", "txt", "nfo"]),
                        ("All Files", &["*"]),
                    ],
                    app.tx_event.clone(),
                );
            }

            let clear_diz = ui.add_enabled(
                !app.is_busy && !app.create_file_id_path.is_empty(),
                egui::Button::new("Clear"),
            ).on_hover_text("Remove attached FILE_ID.DIZ file");
            if clear_diz.clicked() {
                app.create_file_id_path.clear();
            }
        });
    });

    ui.add_space(10.0);

    // Card 3: Action Button
    let has_orig = !app.create_original_path.trim().is_empty();
    let has_patched = !app.create_patched_path.trim().is_empty();
    let has_out = !app.create_output_path.trim().is_empty();
    let can_create = has_orig && has_patched && has_out && !app.is_busy;

    ui.horizontal(|ui| {
        let create_btn = ui.add_enabled(
            can_create,
            egui::Button::new(RichText::new("Create PPF3 Patch").size(15.0).strong())
                .min_size(egui::vec2(180.0, 36.0)),
        ).on_hover_text("Compare differences across CPU cores in parallel and generate the PPF3 patch file.");
        if create_btn.clicked() {
            let orig_path = PathBuf::from(clean_path_str(&app.create_original_path));
            let patched_path = PathBuf::from(clean_path_str(&app.create_patched_path));
            let out_path = PathBuf::from(clean_path_str(&app.create_output_path));

            let mut file_id_data = None;
            let file_id_clean = clean_path_str(&app.create_file_id_path);
            if !file_id_clean.is_empty() {
                match std::fs::read(&file_id_clean) {
                    Ok(data) => file_id_data = Some(data),
                    Err(err) => {
                        app.create_status =
                            Some(Err(format!("Failed to read FILE_ID.DIZ file: {}", err)));
                        return;
                    }
                }
            }

            let options = PpfCreatorOptions {
                description: if app.create_description.trim().is_empty() {
                    ppf_core::DEFAULT_DESCRIPTION.to_string()
                } else {
                    app.create_description.clone()
                },
                image_type: app.create_image_type,
                block_check: app.create_block_check,
                undo_data: app.create_undo_data,
                file_id: file_id_data,
            };

            app.is_busy = true;
            app.busy_operation = "Generating PPF3 patch...".to_string();
            app.progress = 0.0;
            app.progress_msg = "Scanning differences...".to_string();
            app.create_status = None;

            create_patch_async(
                orig_path,
                patched_path,
                out_path,
                options,
                app.tx_event.clone(),
            );
        }
    });

    // Progress Bar (when busy)
    widgets::render_progress(app, ui);

    // Status Banner
    widgets::render_status_banner(app.create_status.as_ref(), ui);
}
