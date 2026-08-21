use crate::views;
use crate::worker::{
    FilePickTarget, HashAlgorithm, InspectTargetTab, WorkerEvent, compute_all_hashes_async,
    compute_hash_async, inspect_patch_async,
};
use egui::{Color32, RichText};
use ppf_core::{ImageType, PatchInfo};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum HashState {
    #[default]
    NotCalculated,
    Calculating,
    Calculated(String),
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    ApplyUndo,
    Create,
    Info,
}

pub fn clean_path_str(path: &str) -> String {
    path.trim().trim_matches(&['\'', '"'][..]).to_string()
}

fn load_app_logo_texture(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let icon_bytes = include_bytes!("../assets/app_logo.png");
    let image = image::load_from_memory(icon_bytes).ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(
        [width as usize, height as usize],
        image.as_flat_samples().as_slice(),
    );
    Some(ctx.load_texture("app_logo", color_image, egui::TextureOptions::LINEAR))
}

pub struct PpfApp {
    pub active_tab: Tab,
    pub tx_event: Sender<WorkerEvent>,
    pub rx_event: Receiver<WorkerEvent>,
    pub logo_texture: Option<egui::TextureHandle>,

    // Busy & Progress state
    pub is_busy: bool,
    pub busy_operation: String,
    pub progress: f32,
    pub progress_msg: String,

    // Help modal state
    pub show_help: bool,

    // Apply & Undo tab state
    pub apply_bin_path: String,
    pub apply_patch_path: String,
    pub apply_in_place: bool,
    pub apply_output_copy_path: String,
    pub apply_patch_info: Option<PatchInfo>,
    pub apply_inspect_error: Option<String>,
    pub action_status: Option<Result<String, String>>,

    // Target image hash state (CRC32, MD5, SHA-1)
    pub hash_crc32: HashState,
    pub hash_md5: HashState,
    pub hash_sha1: HashState,
    pub hash_source_path: String,
    pub hash_copied_feedback: Option<(HashAlgorithm, std::time::Instant)>,

    // Create tab state
    pub create_original_path: String,
    pub create_patched_path: String,
    pub create_output_path: String,
    pub create_description: String,
    pub create_image_type: ImageType,
    pub create_block_check: bool,
    pub create_undo_data: bool,
    pub create_file_id_path: String,
    pub create_status: Option<Result<String, String>>,

    // Info tab state
    pub info_patch_path: String,
    pub info_patch_info: Option<PatchInfo>,
    pub info_inspect_error: Option<String>,
}

impl PpfApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (tx_event, rx_event) = channel();
        let logo_texture = load_app_logo_texture(&cc.egui_ctx);

        Self {
            active_tab: Tab::ApplyUndo,
            tx_event,
            rx_event,
            logo_texture,

            is_busy: false,
            busy_operation: String::new(),
            progress: 0.0,
            progress_msg: String::new(),

            show_help: false,

            apply_bin_path: String::new(),
            apply_patch_path: String::new(),
            apply_in_place: true,
            apply_output_copy_path: String::new(),
            apply_patch_info: None,
            apply_inspect_error: None,
            action_status: None,

            hash_crc32: HashState::NotCalculated,
            hash_md5: HashState::NotCalculated,
            hash_sha1: HashState::NotCalculated,
            hash_source_path: String::new(),
            hash_copied_feedback: None,

            create_original_path: String::new(),
            create_patched_path: String::new(),
            create_output_path: String::new(),
            create_description: ppf_core::DEFAULT_DESCRIPTION.to_string(),
            create_image_type: ImageType::Bin,
            create_block_check: true,
            create_undo_data: false,
            create_file_id_path: String::new(),
            create_status: None,

            info_patch_path: String::new(),
            info_patch_info: None,
            info_inspect_error: None,
        }
    }

    pub fn inspect_patch_if_exists(&self, target_tab: InspectTargetTab) {
        let raw_path = match target_tab {
            InspectTargetTab::ApplyUndo => &self.apply_patch_path,
            InspectTargetTab::Info => &self.info_patch_path,
        };
        let clean = clean_path_str(raw_path);
        if !clean.is_empty() {
            let path = PathBuf::from(clean);
            if path.exists() {
                inspect_patch_async(path, target_tab, self.tx_event.clone());
            }
        }
    }

    fn handle_worker_events(&mut self) {
        while let Ok(event) = self.rx_event.try_recv() {
            match event {
                WorkerEvent::Progress { ratio, message } => {
                    self.progress = ratio;
                    self.progress_msg = message;
                }
                WorkerEvent::InspectComplete {
                    patch_path,
                    target_tab,
                    result,
                } => match target_tab {
                    InspectTargetTab::ApplyUndo => {
                        if std::path::Path::new(&clean_path_str(&self.apply_patch_path))
                            == patch_path
                        {
                            match result {
                                Ok(info) => {
                                    self.apply_patch_info = Some(info);
                                    self.apply_inspect_error = None;
                                }
                                Err(err) => {
                                    self.apply_patch_info = None;
                                    self.apply_inspect_error = Some(err);
                                }
                            }
                        }
                    }
                    InspectTargetTab::Info => {
                        if std::path::Path::new(&clean_path_str(&self.info_patch_path))
                            == patch_path
                        {
                            match result {
                                Ok(info) => {
                                    self.info_patch_info = Some(info);
                                    self.info_inspect_error = None;
                                }
                                Err(err) => {
                                    self.info_patch_info = None;
                                    self.info_inspect_error = Some(err);
                                }
                            }
                        }
                    }
                },
                WorkerEvent::ApplyComplete(result) => {
                    self.is_busy = false;
                    match result {
                        Ok(apply_res) => {
                            if apply_res.was_in_place {
                                self.clear_hashes();
                                self.action_status = Some(Ok(format!(
                                    "Patch applied in-place successfully: '{}'",
                                    apply_res.info.description
                                )));
                            } else {
                                self.action_status = Some(Ok(format!(
                                    "Patch applied to safe copy successfully: '{}'\nSaved to: {}",
                                    apply_res.info.description,
                                    apply_res.target_file.display()
                                )));
                            }
                        }
                        Err(err) => {
                            self.action_status = Some(Err(err));
                        }
                    }
                }
                WorkerEvent::UndoComplete(result) => {
                    self.is_busy = false;
                    match result {
                        Ok(info) => {
                            self.clear_hashes();
                            self.action_status = Some(Ok(format!(
                                "Patch reversed successfully: '{}'",
                                info.description
                            )));
                        }
                        Err(err) => {
                            self.action_status = Some(Err(err));
                        }
                    }
                }
                WorkerEvent::CreateComplete(result) => {
                    self.is_busy = false;
                    match result {
                        Ok(entries) => {
                            self.create_status = Some(Ok(format!(
                                "PPF3 patch created successfully ({} diff records written)",
                                entries
                            )));
                        }
                        Err(err) => {
                            self.create_status = Some(Err(err));
                        }
                    }
                }
                WorkerEvent::HashComplete {
                    path,
                    algorithm,
                    result,
                } => {
                    let current_clean = clean_path_str(&self.apply_bin_path);
                    if std::path::Path::new(&current_clean) == path {
                        let state = match result {
                            Ok(hash) => HashState::Calculated(hash),
                            Err(err) => HashState::Error(err),
                        };
                        match algorithm {
                            HashAlgorithm::Crc32 => self.hash_crc32 = state,
                            HashAlgorithm::Md5 => self.hash_md5 = state,
                            HashAlgorithm::Sha1 => self.hash_sha1 = state,
                        }
                    }
                }
                WorkerEvent::FilePicked { target, path } => {
                    let path_str = path.to_string_lossy().to_string();
                    match target {
                        FilePickTarget::ApplyBin => {
                            self.apply_bin_path = path_str.clone();
                            self.apply_output_copy_path =
                                crate::worker::generate_patched_path(&path)
                                    .to_string_lossy()
                                    .to_string();
                            self.action_status = None;
                            self.reset_hashes_if_path_changed();
                        }
                        FilePickTarget::ApplyOutputCopy => {
                            self.apply_output_copy_path = path_str;
                        }
                        FilePickTarget::ApplyPatch => {
                            self.apply_patch_path = path_str;
                            self.apply_patch_info = None;
                            self.apply_inspect_error = None;
                            self.action_status = None;
                            inspect_patch_async(
                                path,
                                InspectTargetTab::ApplyUndo,
                                self.tx_event.clone(),
                            );
                        }
                        FilePickTarget::CreateOriginal => {
                            self.create_original_path = path_str;
                            self.create_status = None;
                        }
                        FilePickTarget::CreatePatched => {
                            self.create_patched_path = path_str;
                            self.create_status = None;
                        }
                        FilePickTarget::CreateOutput => {
                            self.create_output_path = path_str;
                            self.create_status = None;
                        }
                        FilePickTarget::CreateFileId => {
                            self.create_file_id_path = path_str;
                        }
                        FilePickTarget::InfoPatch => {
                            self.info_patch_path = path_str;
                            self.info_patch_info = None;
                            self.info_inspect_error = None;
                            inspect_patch_async(
                                path,
                                InspectTargetTab::Info,
                                self.tx_event.clone(),
                            );
                        }
                    }
                }
            }
        }
    }

    fn handle_drag_and_drop(&mut self, ctx: &egui::Context) {
        if self.is_busy {
            return;
        }
        let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
        for dropped in dropped_files {
            if let Some(path) = dropped.path {
                let path_str = path.to_string_lossy().to_string();
                let is_ppf = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("ppf"));

                match self.active_tab {
                    Tab::ApplyUndo => {
                        if is_ppf {
                            self.apply_patch_path = path_str;
                            self.apply_patch_info = None;
                            self.apply_inspect_error = None;
                            self.action_status = None;
                            inspect_patch_async(
                                path,
                                InspectTargetTab::ApplyUndo,
                                self.tx_event.clone(),
                            );
                        } else {
                            self.apply_bin_path = path_str;
                            self.apply_output_copy_path =
                                crate::worker::generate_patched_path(&path)
                                    .to_string_lossy()
                                    .to_string();
                            self.action_status = None;
                            self.reset_hashes_if_path_changed();
                        }
                    }
                    Tab::Create => {
                        let is_diz =
                            path.extension()
                                .and_then(|ext| ext.to_str())
                                .is_some_and(|ext| {
                                    ext.eq_ignore_ascii_case("diz")
                                        || ext.eq_ignore_ascii_case("nfo")
                                        || ext.eq_ignore_ascii_case("txt")
                                });

                        if is_ppf {
                            self.create_output_path = path_str;
                            self.create_status = None;
                        } else if is_diz {
                            self.create_file_id_path = path_str;
                        } else if self.create_original_path.is_empty() {
                            self.create_original_path = path_str;
                            self.create_status = None;
                        } else {
                            self.create_patched_path = path_str;
                            self.create_status = None;
                        }
                    }
                    Tab::Info => {
                        self.info_patch_path = path_str;
                        self.info_patch_info = None;
                        self.info_inspect_error = None;
                        inspect_patch_async(path, InspectTargetTab::Info, self.tx_event.clone());
                    }
                }
            }
        }
    }

    pub fn reset_hashes_if_path_changed(&mut self) {
        let clean = clean_path_str(&self.apply_bin_path);
        if clean != self.hash_source_path {
            self.hash_source_path = clean;
            self.hash_crc32 = HashState::NotCalculated;
            self.hash_md5 = HashState::NotCalculated;
            self.hash_sha1 = HashState::NotCalculated;
            self.hash_copied_feedback = None;
        }
    }

    pub fn calculate_hash(&mut self, algorithm: HashAlgorithm) {
        let clean = clean_path_str(&self.apply_bin_path);
        if clean.is_empty() {
            return;
        }
        let path = PathBuf::from(&clean);
        if !path.is_file() {
            let err = "File does not exist or is not a regular file".to_string();
            match algorithm {
                HashAlgorithm::Crc32 => self.hash_crc32 = HashState::Error(err),
                HashAlgorithm::Md5 => self.hash_md5 = HashState::Error(err),
                HashAlgorithm::Sha1 => self.hash_sha1 = HashState::Error(err),
            }
            return;
        }

        self.hash_source_path = clean;
        match algorithm {
            HashAlgorithm::Crc32 => self.hash_crc32 = HashState::Calculating,
            HashAlgorithm::Md5 => self.hash_md5 = HashState::Calculating,
            HashAlgorithm::Sha1 => self.hash_sha1 = HashState::Calculating,
        }

        compute_hash_async(path, algorithm, self.tx_event.clone());
    }

    pub fn calculate_all_hashes(&mut self) {
        let clean = clean_path_str(&self.apply_bin_path);
        if clean.is_empty() {
            return;
        }
        let path = PathBuf::from(&clean);
        if !path.is_file() {
            let err = "File does not exist or is not a regular file".to_string();
            self.hash_crc32 = HashState::Error(err.clone());
            self.hash_md5 = HashState::Error(err.clone());
            self.hash_sha1 = HashState::Error(err);
            return;
        }

        self.hash_source_path = clean;
        self.hash_crc32 = HashState::Calculating;
        self.hash_md5 = HashState::Calculating;
        self.hash_sha1 = HashState::Calculating;

        compute_all_hashes_async(path, self.tx_event.clone());
    }

    pub fn clear_hashes(&mut self) {
        self.hash_crc32 = HashState::NotCalculated;
        self.hash_md5 = HashState::NotCalculated;
        self.hash_sha1 = HashState::NotCalculated;
        self.hash_copied_feedback = None;
    }

    pub fn is_hashing(&self) -> bool {
        self.hash_crc32 == HashState::Calculating
            || self.hash_md5 == HashState::Calculating
            || self.hash_sha1 == HashState::Calculating
    }
}

impl eframe::App for PpfApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_worker_events();
        self.handle_drag_and_drop(ctx);

        if self.is_busy || self.is_hashing() {
            ctx.request_repaint_after(std::time::Duration::from_millis(33));
        }

        // Help dialog window
        if self.show_help {
            views::help::show(&mut self.show_help, ctx, self.logo_texture.as_ref());
        }

        // Top Header and Tab Bar
        egui::TopBottomPanel::top("top_navigation_panel").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if let Some(texture) = &self.logo_texture {
                    ui.image((texture.id(), egui::vec2(22.0, 22.0)));
                }
                ui.heading(
                    RichText::new("PPF Rust Patcher")
                        .strong()
                        .color(Color32::from_rgb(100, 180, 255)),
                );
                ui.label(RichText::new(concat!("v", env!("CARGO_PKG_VERSION"))).weak());

                ui.add_space(20.0);

                let tab_apply = ui.selectable_value(
                    &mut self.active_tab,
                    Tab::ApplyUndo,
                    RichText::new("  Apply & Undo  ").strong(),
                );
                tab_apply.on_hover_text("Apply PPF patches to game disc images or reverse previously applied patches.");

                let tab_create = ui.selectable_value(
                    &mut self.active_tab,
                    Tab::Create,
                    RichText::new("  Create Patch  ").strong(),
                );
                tab_create.on_hover_text("Generate a new PPF3 patch file by comparing original and modified binary files.");

                let tab_info = ui.selectable_value(
                    &mut self.active_tab,
                    Tab::Info,
                    RichText::new("  Info & DIZ  ").strong(),
                );
                tab_info.on_hover_text("Inspect patch metadata, block check status, and embedded FILE_ID.DIZ release information.");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let help_btn = ui.button(RichText::new("Help & Guide").strong());
                    if help_btn.clicked() {
                        self.show_help = true;
                    }
                    help_btn.on_hover_text("Open beginner-friendly guide and documentation explaining how PPF patching works.");
                });
            });
            ui.add_space(6.0);
        });

        // Bottom Footer Panel
        egui::TopBottomPanel::bottom("bottom_footer_panel").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(concat!("PPF Rust Patcher v", env!("CARGO_PKG_VERSION")))
                        .weak()
                        .size(11.0),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.hyperlink_to(
                        RichText::new("https://github.com/plinkr/ppf_rust")
                            .size(11.0)
                            .color(Color32::from_rgb(100, 180, 255)),
                        "https://github.com/plinkr/ppf_rust",
                    );
                    ui.label(RichText::new("GitHub:").weak().size(11.0));
                });
            });
            ui.add_space(4.0);
        });

        // Main Content Area
        egui::CentralPanel::default().show(ctx, |ui| {
            if ctx.input(|i| !i.raw.hovered_files.is_empty()) {
                ui.centered_and_justified(|ui| {
                    ui.heading(
                        RichText::new("Drop files here to load")
                            .color(Color32::from_rgb(100, 200, 255))
                            .strong(),
                    );
                });
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| match self.active_tab {
                    Tab::ApplyUndo => views::apply_undo::show(self, ui),
                    Tab::Create => views::create::show(self, ui),
                    Tab::Info => views::info::show(self, ui),
                });
            }
        });
    }
}
