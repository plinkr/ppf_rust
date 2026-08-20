use ppf_core::{
    PatchInfo, PpfCreatorOptions, apply_patch, create_patch, inspect_patch, undo_patch,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilePickTarget {
    ApplyBin,
    ApplyPatch,
    ApplyOutputCopy,
    CreateOriginal,
    CreatePatched,
    CreateOutput,
    CreateFileId,
    InfoPatch,
}

#[derive(Debug, Clone)]
pub struct ApplyResult {
    pub info: PatchInfo,
    pub target_file: PathBuf,
    pub was_in_place: bool,
}

pub enum WorkerEvent {
    Progress {
        ratio: f32,
        message: String,
    },
    InspectComplete {
        patch_path: PathBuf,
        target_tab: InspectTargetTab,
        result: Result<PatchInfo, String>,
    },
    ApplyComplete(Result<ApplyResult, String>),
    UndoComplete(Result<PatchInfo, String>),
    CreateComplete(Result<usize, String>),
    FilePicked {
        target: FilePickTarget,
        path: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectTargetTab {
    ApplyUndo,
    Info,
}

pub fn generate_patched_path(original_path: &Path) -> PathBuf {
    let parent = original_path.parent().unwrap_or_else(|| Path::new(""));
    let stem = original_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");
    let ext = original_path.extension().and_then(|e| e.to_str());

    let new_file_name = match ext {
        Some(extension) => format!("{}_patched.{}", stem, extension),
        None => format!("{}_patched", stem),
    };

    parent.join(new_file_name)
}

pub fn pick_file_async(
    target: FilePickTarget,
    title: &'static str,
    filters: Vec<(&'static str, &'static [&'static str])>,
    tx: Sender<WorkerEvent>,
) {
    thread::spawn(move || {
        let mut dialog = rfd::FileDialog::new().set_title(title);
        for (name, extensions) in filters {
            dialog = dialog.add_filter(name, extensions);
        }
        if let Some(path) = dialog.pick_file() {
            let _ = tx.send(WorkerEvent::FilePicked { target, path });
        }
    });
}

pub fn save_file_async(
    target: FilePickTarget,
    title: &'static str,
    default_name: &'static str,
    filters: Vec<(&'static str, &'static [&'static str])>,
    tx: Sender<WorkerEvent>,
) {
    thread::spawn(move || {
        let mut dialog = rfd::FileDialog::new()
            .set_title(title)
            .set_file_name(default_name);
        for (name, extensions) in filters {
            dialog = dialog.add_filter(name, extensions);
        }
        if let Some(path) = dialog.save_file() {
            let _ = tx.send(WorkerEvent::FilePicked { target, path });
        }
    });
}

pub fn inspect_patch_async(
    patch_path: PathBuf,
    target_tab: InspectTargetTab,
    tx: Sender<WorkerEvent>,
) {
    thread::spawn(move || {
        let result = inspect_patch(&patch_path).map_err(|err| err.to_string());
        let _ = tx.send(WorkerEvent::InspectComplete {
            patch_path,
            target_tab,
            result,
        });
    });
}

fn create_record_progress_callbacks(
    action: &'static str,
    tx: Sender<WorkerEvent>,
) -> (
    impl Fn(usize) + Send + Sync + 'static,
    impl Fn(usize) + Send + Sync + 'static,
) {
    let total_records = Arc::new(AtomicUsize::new(0));
    let current_records = Arc::new(AtomicUsize::new(0));

    let tx_start = tx.clone();
    let total_start = Arc::clone(&total_records);
    let on_start = move |total: usize| {
        total_start.store(total, Ordering::SeqCst);
        let _ = tx_start.send(WorkerEvent::Progress {
            ratio: 0.0,
            message: format!(
                "Starting {}: 0 / {} records (0.0%)",
                action.to_lowercase(),
                total
            ),
        });
    };

    let total_prog = Arc::clone(&total_records);
    let current_prog = Arc::clone(&current_records);
    let on_progress = move |records: usize| {
        let done = current_prog.fetch_add(records, Ordering::Relaxed) + records;
        let total = total_prog.load(Ordering::Relaxed);
        if total > 0 && (done == total || done.is_multiple_of(200)) {
            let ratio = (done as f32 / total as f32).clamp(0.0, 1.0);
            let _ = tx.send(WorkerEvent::Progress {
                ratio,
                message: format!(
                    "{}: {} / {} records ({:.1}%)",
                    action,
                    done,
                    total,
                    ratio * 100.0
                ),
            });
        }
    };

    (on_start, on_progress)
}

pub fn apply_patch_async(
    patch_path: PathBuf,
    bin_path: PathBuf,
    patch_in_place: bool,
    custom_output: Option<PathBuf>,
    tx: Sender<WorkerEvent>,
) {
    thread::spawn(move || {
        let target_bin = if patch_in_place {
            bin_path
        } else {
            let dest = custom_output.unwrap_or_else(|| generate_patched_path(&bin_path));
            if dest == bin_path {
                let _ = tx.send(WorkerEvent::ApplyComplete(Err(
                    "Safe copy destination path cannot be identical to original binary path"
                        .to_string(),
                )));
                return;
            }
            let _ = tx.send(WorkerEvent::Progress {
                ratio: 0.0,
                message: format!(
                    "Creating backup copy: {}...",
                    dest.file_name()
                        .map(|n| n.to_string_lossy())
                        .unwrap_or_default()
                ),
            });
            if let Err(err) = std::fs::copy(&bin_path, &dest) {
                let _ = tx.send(WorkerEvent::ApplyComplete(Err(format!(
                    "Failed to create safe copy: {}",
                    err
                ))));
                return;
            }
            dest
        };

        let (on_start, on_progress) = create_record_progress_callbacks("Applying", tx.clone());

        let result = apply_patch(
            &patch_path,
            &target_bin,
            Some(&on_start),
            Some(&on_progress),
        )
        .map(|info| ApplyResult {
            info,
            target_file: target_bin,
            was_in_place: patch_in_place,
        })
        .map_err(|err| err.to_string());

        let _ = tx.send(WorkerEvent::ApplyComplete(result));
    });
}

pub fn undo_patch_async(patch_path: PathBuf, bin_path: PathBuf, tx: Sender<WorkerEvent>) {
    thread::spawn(move || {
        let (on_start, on_progress) = create_record_progress_callbacks("Undoing", tx.clone());

        let result = undo_patch(&patch_path, &bin_path, Some(&on_start), Some(&on_progress))
            .map_err(|err| err.to_string());

        let _ = tx.send(WorkerEvent::UndoComplete(result));
    });
}

pub fn create_patch_async(
    original_path: PathBuf,
    patched_path: PathBuf,
    output_path: PathBuf,
    options: PpfCreatorOptions,
    tx: Sender<WorkerEvent>,
) {
    thread::spawn(move || {
        let total_bytes = std::fs::metadata(&original_path)
            .map(|meta| meta.len())
            .unwrap_or(1)
            .max(1) as usize;

        let processed_bytes = Arc::new(AtomicUsize::new(0));
        let tx_progress = tx.clone();
        let processed_clone = Arc::clone(&processed_bytes);

        let progress_cb = move |bytes: usize| {
            let done = processed_clone.fetch_add(bytes, Ordering::Relaxed) + bytes;
            let ratio = (done as f32 / total_bytes as f32).clamp(0.0, 1.0);
            let mb_done = done as f64 / (1024.0 * 1024.0);
            let mb_total = total_bytes as f64 / (1024.0 * 1024.0);
            let _ = tx_progress.send(WorkerEvent::Progress {
                ratio,
                message: format!(
                    "{:.1} MB / {:.1} MB ({:.1}%)",
                    mb_done,
                    mb_total,
                    ratio * 100.0
                ),
            });
        };

        let result = create_patch(
            &original_path,
            &patched_path,
            &output_path,
            &options,
            Some(&progress_cb),
        )
        .map_err(|err| err.to_string());

        let _ = tx.send(WorkerEvent::CreateComplete(result));
    });
}
