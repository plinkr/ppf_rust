use egui::{Color32, RichText, TextureHandle, Window};

pub fn show(open: &mut bool, ctx: &egui::Context, logo: Option<&TextureHandle>) {
    let mut is_open = *open;
    let mut close_requested = false;

    Window::new(RichText::new("PPF Rust Patcher - User Guide & Help").strong())
        .open(&mut is_open)
        .resizable(true)
        .default_width(580.0)
        .default_height(480.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if let Some(texture) = logo {
                        ui.image((texture.id(), egui::vec2(64.0, 64.0)));
                    }
                    ui.heading("Welcome to PPF Rust Patcher!");
                });
                ui.label(
                    RichText::new(
                        "This tool allows you to easily apply, reverse, and create PPF (PlayStation Patch Format) patches for disc images (CD/DVD) and ROM files without needing complex command-line tools.",
                    )
                    .italics(),
                );
                ui.add_space(8.0);

                // Section 1: Overview
                ui.collapsing(RichText::new("1. What is a PPF Patch?").strong(), |ui| {
                    ui.label(
                        "PPF (PlayStation Patch Format) is a binary diff patching standard originally designed for PlayStation CD-ROM images and widely used for translations, game modifications, and fan patches.",
                    );
                    ui.add_space(4.0);
                    ui.label(RichText::new("Supported Versions:").strong());
                    ui.label("- PPF 1.0: Legacy format with simple byte offsets and raw diff records.");
                    ui.label("- PPF 2.0: Introduces a 1024-byte checksum block validation to ensure you are patching the correct file.");
                    ui.label("- PPF 3.0: Modern format with 64-bit offsets (supports multi-gigabyte DVD/Blu-ray images), optional Blockcheck validation, embedded FILE_ID.DIZ release descriptions, and reversible Undo support.");
                });

                ui.add_space(6.0);

                // Section 2: How to Apply a Patch
                ui.collapsing(RichText::new("2. How to Apply a Patch").strong(), |ui| {
                    ui.label(RichText::new("Step-by-step:").strong());
                    ui.label("1. Select your Target Binary Image (.bin, .iso, .img, etc.) or drag & drop it into the window.");
                    ui.label("2. Select your PPF Patch File (.ppf) or drag & drop it into the window.");
                    ui.label("3. Choose your patching mode:");
                    ui.indent("in_place_help", |ui| {
                        ui.label(
                            RichText::new("Patch in-place (Default):")
                                .strong()
                                .color(Color32::from_rgb(100, 180, 255)),
                        );
                        ui.label("  Directly modifies the target file. Fast and requires no extra disk space.");
                        ui.label(
                            RichText::new("Uncheck 'Patch in-place':")
                                .strong()
                                .color(Color32::from_rgb(100, 220, 120)),
                        );
                        ui.label("  Creates a safe copy (e.g. game_patched.bin) leaving your original image completely untouched.");
                    });
                    ui.label("4. Review the Patch Details card (Description, Version, Validation status).");
                    ui.label("5. Click the 'Apply Patch' button and wait for the progress bar to complete.");
                });

                ui.add_space(6.0);

                // Section 3: How to Undo a Patch
                ui.collapsing(RichText::new("3. How to Undo (Reverse) a Patch").strong(), |ui| {
                    ui.label(
                        "If a PPF 3.0 patch was created with 'Undo Data' enabled, you can completely reverse the changes and restore the original binary bytes without needing a fresh game dump.",
                    );
                    ui.add_space(4.0);
                    ui.label("1. Select the modified/patched binary image in the 'Apply & Undo' tab.");
                    ui.label("2. Select the matching .ppf patch file.");
                    ui.label("3. If the patch contains undo data, the 'Undo Patch' button will become active.");
                    ui.label("4. Click 'Undo Patch' to restore the binary to its original state.");
                });

                ui.add_space(6.0);

                // Section 4: How to Create a PPF3 Patch
                ui.collapsing(RichText::new("4. How to Create a PPF3 Patch").strong(), |ui| {
                    ui.label("To create a patch for your translation or mod:");
                    ui.add_space(4.0);
                    ui.label("1. In the 'Create Patch' tab, select your Original Unmodified Binary.");
                    ui.label("2. Select your Modified/Patched Binary. (Note: Both files MUST have the exact same file size in bytes).");
                    ui.label("3. Choose the Output PPF3 Patch File path.");
                    ui.label("4. Configure patch options:");
                    ui.indent("create_options_help", |ui| {
                        ui.label(RichText::new("- Description: Short title or author credit (max 50 chars).").strong());
                        ui.label(RichText::new("- Image Type: Choose 'BIN/RAW' for normal CD/ISO images or 'GI' for Global Image format.").strong());
                        ui.label(RichText::new("- Include Block Check: Highly recommended. Prevents users from applying your patch to the wrong revision/region.").strong());
                        ui.label(RichText::new("- Include Undo Data: Enables users to reverse your patch later.").strong());
                        ui.label(RichText::new("- FILE_ID.DIZ: Optional release text file containing changelogs, credits, or instructions.").strong());
                    });
                    ui.label("5. Click 'Create PPF3 Patch'. The engine uses all CPU cores to compare differences in parallel.");
                });

                ui.add_space(6.0);

                // Section 5: Troubleshooting
                ui.collapsing(RichText::new("5. Common Errors & Troubleshooting").strong(), |ui| {
                    ui.label(
                        RichText::new("Error: Bin file size mismatch")
                            .color(Color32::from_rgb(255, 120, 120))
                            .strong(),
                    );
                    ui.label("  The target binary image size does not match what the patch expects. Make sure you are using an uncompressed raw image (.bin/.iso).");
                    ui.add_space(4.0);

                    ui.label(
                        RichText::new("Error: Block check validation failed")
                            .color(Color32::from_rgb(255, 120, 120))
                            .strong(),
                    );
                    ui.label("  The disc image checksum did not match. You might be using a different game region (e.g. NTSC vs PAL), a bad dump, or a file that has already been patched.");
                    ui.add_space(4.0);

                    ui.label(
                        RichText::new("Error: Undo data not available")
                            .color(Color32::from_rgb(255, 120, 120))
                            .strong(),
                    );
                    ui.label("  This patch is a one-way patch and did not include original restoration data.");
                });

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.hyperlink_to(
                        "GitHub: https://github.com/plinkr/ppf_rust",
                        "https://github.com/plinkr/ppf_rust",
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(RichText::new("Close Guide").strong()).clicked() {
                            close_requested = true;
                        }
                    });
                });
            });
        });

    if close_requested {
        is_open = false;
    }
    *open = is_open;
}
