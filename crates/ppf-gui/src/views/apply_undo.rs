use crate::app::{HashState, PpfApp, clean_path_str};
use crate::views::widgets;
use crate::worker::{
    FilePickTarget, HashAlgorithm, InspectTargetTab, apply_patch_async, pick_file_async,
    save_file_async, undo_patch_async,
};
use egui::{Color32, RichText, Ui};
use ppf_core::{ImageType, PpfVersion};
use std::path::PathBuf;

pub fn show(app: &mut PpfApp, ui: &mut Ui) {
    ui.add_space(8.0);

    // Card 1: File Selection
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.heading("Target Image & Patch Selection");
        });
        ui.add_space(4.0);

        // Binary File Input
        ui.label(RichText::new("Target Binary Image (.bin, .iso, .img):").strong());
        ui.horizontal(|ui| {
            let edit = ui.add(
                egui::TextEdit::singleline(&mut app.apply_bin_path)
                    .hint_text("/path/to/game.bin or drag & drop file here")
                    .desired_width(ui.available_width() - 100.0),
            ).on_hover_text(
                "Path to the disc or binary image to be patched (.bin, .iso, .img, .cue, .raw).\nYou can also drag & drop the file directly into the window.",
            );
            if edit.changed() {
                app.action_status = None;
                app.reset_hashes_if_path_changed();
                if app.apply_output_copy_path.is_empty() && !app.apply_bin_path.trim().is_empty() {
                    let clean = clean_path_str(&app.apply_bin_path);
                    app.apply_output_copy_path = crate::worker::generate_patched_path(
                        std::path::Path::new(&clean),
                    )
                    .to_string_lossy()
                    .to_string();
                }
            }

            let browse_bin = ui.add_enabled(!app.is_busy, egui::Button::new("Browse..."))
                .on_hover_text("Open file chooser to select target binary image");
            if browse_bin.clicked() {
                pick_file_async(
                    FilePickTarget::ApplyBin,
                    "Select Target Binary Image",
                    vec![
                        ("Disc Images", &["bin", "iso", "img", "cue", "raw"]),
                        ("All Files", &["*"]),
                    ],
                    app.tx_event.clone(),
                );
            }
        });

        ui.add_space(4.0);

        // Patch in-place checkbox
        ui.checkbox(
            &mut app.apply_in_place,
            RichText::new("Patch in-place (modify original file directly)").strong(),
        ).on_hover_text(
            "Checked: Directly modifies your target binary file (fast, requires no extra disk space).\nUnchecked: Creates a safe copy (e.g. game_patched.bin) leaving the original file untouched.",
        );

        if !app.apply_in_place {
            ui.add_space(2.0);
            ui.indent("safe_copy_indent", |ui| {
                ui.label(
                    RichText::new("Safe Copy Output Path:")
                        .color(Color32::from_rgb(100, 220, 120))
                        .strong(),
                );
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut app.apply_output_copy_path)
                            .hint_text("/path/to/game_patched.bin")
                            .desired_width(ui.available_width() - 100.0),
                    ).on_hover_text("Destination where the patched copy will be saved.");

                    let browse_out =
                        ui.add_enabled(!app.is_busy, egui::Button::new("Change..."))
                            .on_hover_text("Choose a custom destination for the patched copy");
                    if browse_out.clicked() {
                        save_file_async(
                            FilePickTarget::ApplyOutputCopy,
                            "Select Destination for Patched Copy",
                            "game_patched.bin",
                            vec![("Disc Images", &["bin", "iso", "img"]), ("All Files", &["*"])],
                            app.tx_event.clone(),
                        );
                    }
                });
            });
        }

        // Checksums & Hashes Collapsible Section
        ui.add_space(4.0);
        render_hash_collapsible(app, ui);

        ui.add_space(8.0);

        // Patch File Input
        ui.label(RichText::new("PPF Patch File (.ppf):").strong());
        ui.horizontal(|ui| {
            let edit = ui.add(
                egui::TextEdit::singleline(&mut app.apply_patch_path)
                    .hint_text("/path/to/patch.ppf or drag & drop file here")
                    .desired_width(ui.available_width() - 100.0),
            ).on_hover_text(
                "Path to the PPF patch file (PPF 1.0, 2.0, or 3.0).\nYou can also drag & drop the file directly into the window.",
            );
            if edit.changed() {
                app.apply_patch_info = None;
                app.apply_inspect_error = None;
                app.action_status = None;
                app.inspect_patch_if_exists(InspectTargetTab::ApplyUndo);
            }

            let browse_patch = ui.add_enabled(!app.is_busy, egui::Button::new("Browse..."))
                .on_hover_text("Open file chooser to select PPF patch file");
            if browse_patch.clicked() {
                pick_file_async(
                    FilePickTarget::ApplyPatch,
                    "Select PPF Patch File",
                    vec![("PPF Patches", &["ppf"]), ("All Files", &["*"])],
                    app.tx_event.clone(),
                );
            }
        });
    });

    ui.add_space(10.0);

    // Card 2: Patch Metadata
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.heading("Patch Details & Validation");
        ui.add_space(4.0);

        if let Some(info) = &app.apply_patch_info {
            egui::Grid::new("apply_patch_info_grid")
                .num_columns(2)
                .spacing([16.0, 6.0])
                .show(ui, |ui| {
                    ui.label("Format Version:").on_hover_text("PPF specification version used to create this patch.");
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

                    ui.label("Description:").on_hover_text("Title, credit, or description embedded in the patch header.");
                    ui.label(RichText::new(&info.description).italics());
                    ui.end_row();

                    if info.version == PpfVersion::V3 {
                        ui.label("Image Type:").on_hover_text("Expected disc image layout (BIN RAW offset 0x9320 vs GI Global Image offset 0x80A0).");
                        let img_type_str = match info.image_type {
                            ImageType::Bin => "BIN (Standard RAW / ISO)",
                            ImageType::Gi => "GI (Global Image / PrimoDVD)",
                        };
                        ui.label(img_type_str);
                        ui.end_row();

                        ui.label("Block Validation:").on_hover_text("1024-byte checksum verification. Guarantees the patch matches your exact original dump.");
                        if info.block_check {
                            ui.label(
                                RichText::new("Enabled (Protects against wrong game dump)")
                                    .color(Color32::from_rgb(100, 220, 120)),
                            );
                        } else {
                            ui.label(
                                RichText::new("Disabled (No checksum check)")
                                    .color(Color32::from_rgb(200, 200, 200)),
                            );
                        }
                        ui.end_row();

                        ui.label("Undo Data:").on_hover_text("Original byte restoration data. Allows reversing this patch later.");
                        if info.has_undo {
                            ui.label(
                                RichText::new("Available (This patch can be undone later)")
                                    .color(Color32::from_rgb(100, 220, 120))
                                    .strong(),
                            );
                        } else {
                            ui.label(
                                RichText::new("Not available (One-way permanent patch)")
                                    .color(Color32::from_rgb(220, 160, 80)),
                            );
                        }
                        ui.end_row();
                    }

                    ui.label("FILE_ID.DIZ:").on_hover_text("Optional release notes or BBS info embedded in patch trailer.");
                    if info.file_id.is_some() {
                        ui.label(RichText::new("Embedded (View in Info & DIZ tab)").color(Color32::from_rgb(100, 220, 120)));
                    } else {
                        ui.label(RichText::new("None").color(Color32::from_rgb(160, 160, 160)));
                    }
                    ui.end_row();
                });
        } else if let Some(err) = &app.apply_inspect_error {
            ui.colored_label(
                Color32::from_rgb(255, 100, 100),
                format!("Failed to parse patch: {}", err),
            );
        } else {
            ui.label(
                RichText::new("Select or drag & drop a .ppf patch file above to inspect its metadata.")
                    .color(Color32::from_rgb(160, 160, 160)),
            );
        }
    });

    ui.add_space(10.0);

    // Card 3: Action Buttons
    let has_bin = !app.apply_bin_path.trim().is_empty();
    let has_patch = !app.apply_patch_path.trim().is_empty() && app.apply_patch_info.is_some();
    let is_busy = app.is_busy || app.is_hashing();
    let can_apply = has_bin && has_patch && !is_busy;
    let can_undo = can_apply
        && app
            .apply_patch_info
            .as_ref()
            .is_some_and(|info| info.has_undo);

    ui.horizontal(|ui| {
        let apply_btn = ui.add_enabled(
            can_apply,
            egui::Button::new(RichText::new("Apply Patch").size(15.0).strong())
                .min_size(egui::vec2(140.0, 36.0)),
        ).on_hover_text(
            if app.apply_in_place {
                "Apply patch diff records directly into the target binary file in-place."
            } else {
                "Create a safe copy of the target binary and apply the patch to the copy."
            },
        );
        if apply_btn.clicked() {
            let patch_p = PathBuf::from(clean_path_str(&app.apply_patch_path));
            let bin_p = PathBuf::from(clean_path_str(&app.apply_bin_path));
            let custom_out = if !app.apply_in_place && !app.apply_output_copy_path.trim().is_empty() {
                Some(PathBuf::from(clean_path_str(&app.apply_output_copy_path)))
            } else {
                None
            };

            app.is_busy = true;
            app.busy_operation = if app.apply_in_place {
                "Applying patch in-place...".to_string()
            } else {
                "Creating safe copy and applying patch...".to_string()
            };
            app.progress = 0.0;
            app.progress_msg = "Preparing binary...".to_string();
            app.action_status = None;

            apply_patch_async(
                patch_p,
                bin_p,
                app.apply_in_place,
                custom_out,
                app.tx_event.clone(),
            );
        }

        let undo_btn = ui.add_enabled(
            can_undo,
            egui::Button::new(RichText::new("Undo Patch").size(15.0).strong())
                .min_size(egui::vec2(140.0, 36.0)),
        ).on_hover_text(
            "Reverses previously applied PPF3 patch changes and restores original bytes.\n(Requires a PPF3 patch with undo data enabled).",
        );
        if undo_btn.clicked() {
            app.is_busy = true;
            app.busy_operation = "Undoing patch...".to_string();
            app.progress = 0.0;
            app.progress_msg = "Restoring binary...".to_string();
            app.action_status = None;

            undo_patch_async(
                PathBuf::from(clean_path_str(&app.apply_patch_path)),
                PathBuf::from(clean_path_str(&app.apply_bin_path)),
                app.tx_event.clone(),
            );
        }
    });

    // Progress Bar (when busy)
    widgets::render_progress(app, ui);

    // Status / Result Banner
    widgets::render_status_banner(app.action_status.as_ref(), ui);
}

fn render_hash_collapsible(app: &mut PpfApp, ui: &mut Ui) {
    egui::CollapsingHeader::new(
        RichText::new("Image Checksums & Hashes (CRC32, MD5, SHA-1)")
            .color(Color32::from_rgb(170, 200, 240))
            .strong(),
    )
    .default_open(false)
    .show(ui, |ui| {
        ui.add_space(2.0);
        let clean = clean_path_str(&app.apply_bin_path);
        let file_path = std::path::Path::new(&clean);
        let file_valid = !clean.is_empty() && file_path.is_file();

        if !file_valid {
            ui.label(
                RichText::new(
                    "Select a valid target binary image above to compute or verify checksums.",
                )
                .size(12.0)
                .color(Color32::from_rgb(160, 160, 160))
                .italics(),
            );
            return;
        }

        ui.label(
            RichText::new(
                "Generate integrity hashes of the target image (runs concurrently in background):",
            )
            .size(12.0)
            .weak(),
        );
        ui.add_space(4.0);

        let mut trigger_calc: Option<HashAlgorithm> = None;
        let mut trigger_copy: Option<(HashAlgorithm, String)> = None;

        egui::Grid::new("image_hashes_table")
            .num_columns(3)
            .spacing([14.0, 6.0])
            .show(ui, |ui| {
                for algo in [
                    HashAlgorithm::Crc32,
                    HashAlgorithm::Md5,
                    HashAlgorithm::Sha1,
                ] {
                    render_hash_row(ui, app, algo, &mut trigger_calc, &mut trigger_copy);
                    ui.end_row();
                }
            });

        ui.add_space(6.0);

        // Action Buttons Row
        ui.horizontal(|ui| {
            let is_busy = app.is_busy;
            let is_hashing = app.is_hashing();

            let calc_all_btn = ui.add_enabled(
                !is_busy && !is_hashing,
                egui::Button::new(RichText::new("Calculate All Hashes").strong()),
            ).on_hover_text(
                "Computes CRC32, MD5, and SHA-1 simultaneously using background worker threads.",
            );
            if calc_all_btn.clicked() {
                app.calculate_all_hashes();
            }

            let has_any_result = app.hash_crc32 != HashState::NotCalculated
                || app.hash_md5 != HashState::NotCalculated
                || app.hash_sha1 != HashState::NotCalculated;

            if has_any_result {
                let clear_btn = ui
                    .add_enabled(!is_hashing, egui::Button::new("Clear Hashes"))
                    .on_hover_text("Clear all calculated hash values.");
                if clear_btn.clicked() {
                    app.clear_hashes();
                }
            }
        });

        if let Some((algo, text)) = trigger_copy {
            ui.output_mut(|o| o.copied_text = text);
            app.hash_copied_feedback = Some((algo, std::time::Instant::now()));
        }

        if let Some(algo) = trigger_calc {
            app.calculate_hash(algo);
        }
    });
}

fn render_hash_row(
    ui: &mut Ui,
    app: &PpfApp,
    algorithm: HashAlgorithm,
    trigger_calc: &mut Option<HashAlgorithm>,
    trigger_copy: &mut Option<(HashAlgorithm, String)>,
) {
    let label = algorithm.name();
    let tooltip = algorithm.description();

    // Column 1: Label
    ui.label(RichText::new(format!("{}:", label)).strong())
        .on_hover_text(tooltip);

    // Column 2: Status / Hash Output
    let state = match algorithm {
        HashAlgorithm::Crc32 => &app.hash_crc32,
        HashAlgorithm::Md5 => &app.hash_md5,
        HashAlgorithm::Sha1 => &app.hash_sha1,
    };

    match state {
        HashState::NotCalculated => {
            ui.label(
                RichText::new("Not calculated")
                    .color(Color32::from_rgb(140, 140, 140))
                    .italics(),
            );
        }
        HashState::Calculating => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(
                    RichText::new("Calculating...")
                        .color(Color32::from_rgb(255, 200, 100))
                        .italics(),
                );
            });
        }
        HashState::Calculated(hash) => {
            let mut text = hash.clone();
            let width = match algorithm {
                HashAlgorithm::Crc32 => 90.0,
                HashAlgorithm::Md5 => 230.0,
                HashAlgorithm::Sha1 => 290.0,
            };
            ui.add(
                egui::TextEdit::singleline(&mut text)
                    .font(egui::TextStyle::Monospace)
                    .min_size(egui::vec2(width, 20.0))
                    .interactive(true),
            );
        }
        HashState::Error(err) => {
            ui.colored_label(Color32::from_rgb(255, 100, 100), format!("Error: {}", err));
        }
    }

    // Column 3: Action Buttons
    ui.horizontal(|ui| match state {
        HashState::NotCalculated => {
            let btn = ui
                .add_enabled(
                    !app.is_busy,
                    egui::Button::new(format!("Generate {}", label)),
                )
                .on_hover_text(format!("Compute {} checksum in background thread.", label));
            if btn.clicked() {
                *trigger_calc = Some(algorithm);
            }
        }
        HashState::Calculating => {
            ui.add_enabled(false, egui::Button::new("Working..."));
        }
        HashState::Calculated(hash) => {
            let is_copied = app
                .hash_copied_feedback
                .as_ref()
                .is_some_and(|(algo, instant)| {
                    *algo == algorithm && instant.elapsed().as_secs() < 2
                });
            let copy_label = if is_copied {
                "✔ Copied!"
            } else {
                "📋 Copy"
            };
            let copy_btn = ui
                .button(copy_label)
                .on_hover_text("Copy hash to clipboard");
            if copy_btn.clicked() {
                *trigger_copy = Some((algorithm, hash.clone()));
            }

            let recalc_btn = ui
                .add_enabled(!app.is_busy, egui::Button::new("🔄"))
                .on_hover_text(format!("Recalculate {} checksum", label));
            if recalc_btn.clicked() {
                *trigger_calc = Some(algorithm);
            }
        }
        HashState::Error(_) => {
            let retry_btn = ui.add_enabled(!app.is_busy, egui::Button::new("Retry"));
            if retry_btn.clicked() {
                *trigger_calc = Some(algorithm);
            }
        }
    });
}
