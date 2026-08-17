use crate::core::{ImageType, PpfError, PpfVersion};
use crate::parser::PpfFile;
use memmap2::MmapMut;
use std::fs::OpenOptions;
use std::path::Path;

/// Summary metadata and status extracted from a PPF patch header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchInfo {
    pub version: PpfVersion,
    pub description: String,
    pub file_id: Option<String>,
    pub image_type: ImageType,
    pub has_undo: bool,
    pub block_check: bool,
}

/// Inspects a PPF patch file and returns its header metadata without modifying any binary.
pub fn inspect_patch(patch_path: impl AsRef<Path>) -> Result<PatchInfo, PpfError> {
    let ppf = PpfFile::open(patch_path)?;
    Ok(PatchInfo {
        version: ppf.header.version,
        description: ppf.header.description,
        file_id: ppf.header.file_id,
        image_type: ppf.header.image_type,
        has_undo: ppf.header.has_undo,
        block_check: ppf.header.block_check,
    })
}

/// Applies a PPF patch (v1.0, v2.0, or v3.0) to a target binary file.
///
/// # Arguments
/// * `patch_path` - Path to the PPF patch file.
/// * `bin_path` - Path to the target binary file to be modified in-place.
/// * `on_start` - Optional callback invoked with the total number of records before modification.
/// * `progress_callback` - Optional callback invoked with the number of records applied (e.g. 1 per record).
pub fn apply_patch(
    patch_path: impl AsRef<Path>,
    bin_path: impl AsRef<Path>,
    on_start: Option<&dyn Fn(usize)>,
    progress_callback: Option<&dyn Fn(usize)>,
) -> Result<PatchInfo, PpfError> {
    process_patch(patch_path, bin_path, false, on_start, progress_callback)
}

/// Reverses/undoes a PPF3 patch from a previously patched binary file.
///
/// Requires the patch to contain undo data (`has_undo == true`).
///
/// # Arguments
/// * `patch_path` - Path to the PPF patch file.
/// * `bin_path` - Path to the binary file to be restored in-place.
/// * `on_start` - Optional callback invoked with the total number of records before modification.
/// * `progress_callback` - Optional callback invoked with the number of records restored (e.g. 1 per record).
pub fn undo_patch(
    patch_path: impl AsRef<Path>,
    bin_path: impl AsRef<Path>,
    on_start: Option<&dyn Fn(usize)>,
    progress_callback: Option<&dyn Fn(usize)>,
) -> Result<PatchInfo, PpfError> {
    process_patch(patch_path, bin_path, true, on_start, progress_callback)
}

fn process_patch(
    patch_path: impl AsRef<Path>,
    bin_path: impl AsRef<Path>,
    is_undo: bool,
    on_start: Option<&dyn Fn(usize)>,
    progress_callback: Option<&dyn Fn(usize)>,
) -> Result<PatchInfo, PpfError> {
    let ppf = PpfFile::open(patch_path)?;

    if is_undo && !ppf.header.has_undo {
        return Err(PpfError::UndoNotAvailable);
    }

    let bin_file = OpenOptions::new().read(true).write(true).open(bin_path)?;
    let bin_len = bin_file.metadata()?.len();

    if let Some(expected_len) = ppf.header.original_bin_length
        && expected_len as u64 != bin_len
    {
        return Err(PpfError::BinSizeMismatch {
            expected: expected_len as u64,
            actual: bin_len,
        });
    }

    let mut bin_mmap = unsafe { MmapMut::map_mut(&bin_file)? };
    bin_mmap.advise(memmap2::Advice::Sequential)?;

    if let Some(expected_block) = &ppf.header.block_data {
        let block_offset = if ppf.header.image_type == ImageType::Gi {
            0x80A0usize
        } else {
            0x9320usize
        };

        if bin_len < (block_offset + 1024) as u64 {
            return Err(PpfError::BlockCheckFailed);
        }

        if expected_block.as_slice() != &bin_mmap[block_offset..block_offset + 1024] {
            return Err(PpfError::BlockCheckFailed);
        }
    }

    if let Some(start_cb) = on_start {
        let total = ppf.count_records()?;
        start_cb(total);
    }

    for record_res in ppf.records() {
        let record = record_res?;
        let len = record.length as usize;

        let end_offset = record
            .offset
            .checked_add(record.length as u64)
            .filter(|&end| end <= bin_len)
            .ok_or(PpfError::OffsetOutOfBounds {
                offset: record.offset,
                length: len,
                bin_size: bin_len,
            })? as usize;

        let offset = record.offset as usize;

        let data_to_write = if is_undo {
            record.undo_data.ok_or(PpfError::UndoNotAvailable)?
        } else {
            record.data
        };

        bin_mmap[offset..end_offset].copy_from_slice(data_to_write);

        if let Some(cb) = progress_callback {
            cb(1);
        }
    }

    bin_mmap.flush()?;

    Ok(PatchInfo {
        version: ppf.header.version,
        description: ppf.header.description,
        file_id: ppf.header.file_id,
        image_type: ppf.header.image_type,
        has_undo: ppf.header.has_undo,
        block_check: ppf.header.block_check,
    })
}
