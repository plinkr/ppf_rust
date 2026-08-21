use crc32fast::Hasher as Crc32Hasher;
use md5::{Digest, Md5};
use ppf_core::{
    PatchInfo, PpfCreatorOptions, apply_patch, create_patch, inspect_patch, undo_patch,
};
use sha1::Sha1;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    Crc32,
    Md5,
    Sha1,
}

impl HashAlgorithm {
    pub fn name(&self) -> &'static str {
        match self {
            HashAlgorithm::Crc32 => "CRC32",
            HashAlgorithm::Md5 => "MD5",
            HashAlgorithm::Sha1 => "SHA-1",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            HashAlgorithm::Crc32 => {
                "32-bit cyclic redundancy checksum (Redump and No-Intro standard)."
            }
            HashAlgorithm::Md5 => "128-bit MD5 cryptographic hash.",
            HashAlgorithm::Sha1 => "160-bit SHA-1 cryptographic hash.",
        }
    }
}

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
    HashComplete {
        path: PathBuf,
        algorithm: HashAlgorithm,
        result: Result<String, String>,
    },
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

pub fn compute_file_hash(path: &Path, algorithm: HashAlgorithm) -> Result<String, String> {
    let file = File::open(path).map_err(|err| format!("Failed to open file: {}", err))?;
    let mut reader = BufReader::with_capacity(256 * 1024, file);
    let mut buffer = [0u8; 256 * 1024];

    match algorithm {
        HashAlgorithm::Crc32 => {
            let mut hasher = Crc32Hasher::new();
            loop {
                let n = reader.read(&mut buffer).map_err(|e| e.to_string())?;
                if n == 0 {
                    break;
                }
                hasher.update(&buffer[..n]);
            }
            Ok(format!("{:08X}", hasher.finalize()))
        }
        HashAlgorithm::Md5 => {
            let mut hasher = Md5::new();
            loop {
                let n = reader.read(&mut buffer).map_err(|e| e.to_string())?;
                if n == 0 {
                    break;
                }
                hasher.update(&buffer[..n]);
            }
            Ok(format!("{:032x}", hasher.finalize()))
        }
        HashAlgorithm::Sha1 => {
            let mut hasher = Sha1::new();
            loop {
                let n = reader.read(&mut buffer).map_err(|e| e.to_string())?;
                if n == 0 {
                    break;
                }
                hasher.update(&buffer[..n]);
            }
            Ok(format!("{:040x}", hasher.finalize()))
        }
    }
}

pub struct ImageHashes {
    pub crc32: String,
    pub md5: String,
    pub sha1: String,
}

pub fn compute_all_file_hashes(path: &Path) -> Result<ImageHashes, String> {
    let file = File::open(path).map_err(|err| format!("Failed to open file: {}", err))?;
    let mut reader = BufReader::with_capacity(256 * 1024, file);
    let mut buffer = [0u8; 256 * 1024];

    let mut crc32_hasher = Crc32Hasher::new();
    let mut md5_hasher = Md5::new();
    let mut sha1_hasher = Sha1::new();

    loop {
        let n = reader.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        let chunk = &buffer[..n];
        crc32_hasher.update(chunk);
        md5_hasher.update(chunk);
        sha1_hasher.update(chunk);
    }

    Ok(ImageHashes {
        crc32: format!("{:08X}", crc32_hasher.finalize()),
        md5: format!("{:032x}", md5_hasher.finalize()),
        sha1: format!("{:040x}", sha1_hasher.finalize()),
    })
}

pub fn compute_hash_async(path: PathBuf, algorithm: HashAlgorithm, tx: Sender<WorkerEvent>) {
    thread::spawn(move || {
        let result = compute_file_hash(&path, algorithm);
        let _ = tx.send(WorkerEvent::HashComplete {
            path,
            algorithm,
            result,
        });
    });
}

pub fn compute_all_hashes_async(path: PathBuf, tx: Sender<WorkerEvent>) {
    thread::spawn(move || match compute_all_file_hashes(&path) {
        Ok(hashes) => {
            let _ = tx.send(WorkerEvent::HashComplete {
                path: path.clone(),
                algorithm: HashAlgorithm::Crc32,
                result: Ok(hashes.crc32),
            });
            let _ = tx.send(WorkerEvent::HashComplete {
                path: path.clone(),
                algorithm: HashAlgorithm::Md5,
                result: Ok(hashes.md5),
            });
            let _ = tx.send(WorkerEvent::HashComplete {
                path,
                algorithm: HashAlgorithm::Sha1,
                result: Ok(hashes.sha1),
            });
        }
        Err(err) => {
            let _ = tx.send(WorkerEvent::HashComplete {
                path: path.clone(),
                algorithm: HashAlgorithm::Crc32,
                result: Err(err.clone()),
            });
            let _ = tx.send(WorkerEvent::HashComplete {
                path: path.clone(),
                algorithm: HashAlgorithm::Md5,
                result: Err(err.clone()),
            });
            let _ = tx.send(WorkerEvent::HashComplete {
                path,
                algorithm: HashAlgorithm::Sha1,
                result: Err(err),
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::mpsc::channel;
    use tempfile::NamedTempFile;

    #[test]
    fn test_compute_file_hashes_quick_brown_fox() {
        let mut file = NamedTempFile::new().expect("create temp file");
        file.write_all(b"The quick brown fox jumps over the lazy dog")
            .expect("write to temp file");
        let path = file.path();

        let crc = compute_file_hash(path, HashAlgorithm::Crc32).expect("compute crc32");
        assert_eq!(crc, "414FA339");

        let md5 = compute_file_hash(path, HashAlgorithm::Md5).expect("compute md5");
        assert_eq!(md5, "9e107d9d372bb6826bd81d3542a419d6");

        let sha1 = compute_file_hash(path, HashAlgorithm::Sha1).expect("compute sha1");
        assert_eq!(sha1, "2fd4e1c67a2d28fced849ee1bb76e7391b93eb12");
    }

    #[test]
    fn test_compute_all_file_hashes_single_pass() {
        let mut file = NamedTempFile::new().expect("create temp file");
        file.write_all(b"The quick brown fox jumps over the lazy dog")
            .expect("write to temp file");
        let path = file.path();

        let hashes = compute_all_file_hashes(path).expect("compute all hashes");
        assert_eq!(hashes.crc32, "414FA339");
        assert_eq!(hashes.md5, "9e107d9d372bb6826bd81d3542a419d6");
        assert_eq!(hashes.sha1, "2fd4e1c67a2d28fced849ee1bb76e7391b93eb12");
    }

    #[test]
    fn test_compute_file_hashes_empty_file() {
        let file = NamedTempFile::new().expect("create temp file");
        let path = file.path();

        let crc = compute_file_hash(path, HashAlgorithm::Crc32).expect("compute crc32");
        assert_eq!(crc, "00000000");

        let md5 = compute_file_hash(path, HashAlgorithm::Md5).expect("compute md5");
        assert_eq!(md5, "d41d8cd98f00b204e9800998ecf8427e");

        let sha1 = compute_file_hash(path, HashAlgorithm::Sha1).expect("compute sha1");
        assert_eq!(sha1, "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    }

    #[test]
    fn test_compute_file_hash_nonexistent_file() {
        let non_existent = Path::new("/path/that/does/not/exist_12345.bin");
        let result = compute_file_hash(non_existent, HashAlgorithm::Crc32);
        assert!(result.is_err());

        let all_result = compute_all_file_hashes(non_existent);
        assert!(all_result.is_err());
    }

    #[test]
    fn test_compute_all_hashes_async() {
        let mut file = NamedTempFile::new().expect("create temp file");
        file.write_all(b"The quick brown fox jumps over the lazy dog")
            .expect("write");
        let path = file.path().to_path_buf();

        let (tx, rx) = channel();

        compute_all_hashes_async(path.clone(), tx);

        let mut received = 0;
        let mut crc_val = None;
        let mut md5_val = None;
        let mut sha1_val = None;

        while received < 3 {
            if let Ok(WorkerEvent::HashComplete {
                path: p,
                algorithm,
                result,
            }) = rx.recv()
            {
                assert_eq!(p, path);
                let val = result.expect("hash success");
                match algorithm {
                    HashAlgorithm::Crc32 => crc_val = Some(val),
                    HashAlgorithm::Md5 => md5_val = Some(val),
                    HashAlgorithm::Sha1 => sha1_val = Some(val),
                }
                received += 1;
            }
        }

        assert_eq!(crc_val.as_deref(), Some("414FA339"));
        assert_eq!(md5_val.as_deref(), Some("9e107d9d372bb6826bd81d3542a419d6"));
        assert_eq!(
            sha1_val.as_deref(),
            Some("2fd4e1c67a2d28fced849ee1bb76e7391b93eb12")
        );
    }
}
