use crate::core::{ImageType, PpfError, PpfHeader};
#[cfg(feature = "mmap")]
use crate::parser::PpfFile;
use crate::parser::PpfView;
#[cfg(feature = "mmap")]
use memmap2::MmapMut;
#[cfg(feature = "mmap")]
use std::fs::OpenOptions;
#[cfg(feature = "mmap")]
use std::path::Path;

/// Alias for header metadata returned when inspecting or applying a patch.
pub type PatchInfo = PpfHeader;

/// Inspects a PPF patch from an in-memory slice and returns its header metadata.
#[inline]
pub fn inspect_patch_slice(patch_data: &[u8]) -> Result<PatchInfo, PpfError> {
    let view = PpfView::parse(patch_data)?;
    Ok(view.header)
}

/// Inspects a PPF patch file and returns its header metadata without modifying any binary.
#[cfg(feature = "mmap")]
#[inline]
pub fn inspect_patch(patch_path: impl AsRef<Path>) -> Result<PatchInfo, PpfError> {
    let ppf = PpfFile::open(patch_path)?;
    Ok(ppf.header)
}

/// Applies a PPF patch (v1.0, v2.0, or v3.0) to an in-memory mutable binary slice.
///
/// # Arguments
/// * `patch_data` - Byte slice containing the PPF patch.
/// * `target_data` - Target binary slice to be modified in-place.
/// * `on_start` - Optional callback invoked with the total number of records before modification.
/// * `progress_callback` - Optional callback invoked with the number of records applied (e.g. 1 per record).
pub fn apply_patch_slice(
    patch_data: &[u8],
    target_data: &mut [u8],
    on_start: Option<&dyn Fn(usize)>,
    progress_callback: Option<&dyn Fn(usize)>,
) -> Result<PatchInfo, PpfError> {
    process_patch_slice(patch_data, target_data, false, on_start, progress_callback)
}

/// Reverses/undoes a PPF3 patch on an in-memory mutable binary slice.
///
/// Requires the patch to contain undo data (`has_undo == true`).
///
/// # Arguments
/// * `patch_data` - Byte slice containing the PPF patch.
/// * `target_data` - Target binary slice to be restored in-place.
/// * `on_start` - Optional callback invoked with the total number of records before modification.
/// * `progress_callback` - Optional callback invoked with the number of records restored (e.g. 1 per record).
pub fn undo_patch_slice(
    patch_data: &[u8],
    target_data: &mut [u8],
    on_start: Option<&dyn Fn(usize)>,
    progress_callback: Option<&dyn Fn(usize)>,
) -> Result<PatchInfo, PpfError> {
    process_patch_slice(patch_data, target_data, true, on_start, progress_callback)
}

fn process_patch_slice(
    patch_data: &[u8],
    target_data: &mut [u8],
    is_undo: bool,
    on_start: Option<&dyn Fn(usize)>,
    progress_callback: Option<&dyn Fn(usize)>,
) -> Result<PatchInfo, PpfError> {
    let view = PpfView::parse(patch_data)?;

    if is_undo && !view.header.has_undo {
        return Err(PpfError::UndoNotAvailable);
    }

    let bin_len = target_data.len() as u64;

    if let Some(expected_len) = view.header.original_bin_length
        && expected_len as u64 != bin_len
    {
        return Err(PpfError::BinSizeMismatch {
            expected: expected_len as u64,
            actual: bin_len,
        });
    }

    if !is_undo && let Some(expected_block) = &view.header.block_data {
        let block_offset = if view.header.image_type == ImageType::Gi {
            0x80A0usize
        } else {
            0x9320usize
        };

        if bin_len < (block_offset + 1024) as u64 {
            return Err(PpfError::BlockCheckFailed);
        }

        if expected_block.as_slice() != &target_data[block_offset..block_offset + 1024] {
            return Err(PpfError::BlockCheckFailed);
        }
    }

    if let Some(start_cb) = on_start {
        let total = view.count_records()?;
        start_cb(total);
    }

    for record_res in view.records() {
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

        target_data[offset..end_offset].copy_from_slice(data_to_write);

        if let Some(cb) = progress_callback {
            cb(1);
        }
    }

    Ok(view.header)
}

/// Applies a PPF patch (v1.0, v2.0, or v3.0) to a target binary file.
///
/// # Arguments
/// * `patch_path` - Path to the PPF patch file.
/// * `bin_path` - Path to the target binary file to be modified in-place.
/// * `on_start` - Optional callback invoked with the total number of records before modification.
/// * `progress_callback` - Optional callback invoked with the number of records applied (e.g. 1 per record).
#[cfg(feature = "mmap")]
#[inline]
pub fn apply_patch(
    patch_path: impl AsRef<Path>,
    bin_path: impl AsRef<Path>,
    on_start: Option<&dyn Fn(usize)>,
    progress_callback: Option<&dyn Fn(usize)>,
) -> Result<PatchInfo, PpfError> {
    process_patch_file(patch_path, bin_path, false, on_start, progress_callback)
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
#[cfg(feature = "mmap")]
#[inline]
pub fn undo_patch(
    patch_path: impl AsRef<Path>,
    bin_path: impl AsRef<Path>,
    on_start: Option<&dyn Fn(usize)>,
    progress_callback: Option<&dyn Fn(usize)>,
) -> Result<PatchInfo, PpfError> {
    process_patch_file(patch_path, bin_path, true, on_start, progress_callback)
}

#[cfg(feature = "mmap")]
fn process_patch_file(
    patch_path: impl AsRef<Path>,
    bin_path: impl AsRef<Path>,
    is_undo: bool,
    on_start: Option<&dyn Fn(usize)>,
    progress_callback: Option<&dyn Fn(usize)>,
) -> Result<PatchInfo, PpfError> {
    let ppf = PpfFile::open(patch_path)?;

    let bin_file = OpenOptions::new().read(true).write(true).open(bin_path)?;
    let mut bin_mmap = unsafe { MmapMut::map_mut(&bin_file)? };
    #[cfg(unix)]
    bin_mmap.advise(memmap2::Advice::Sequential)?;

    let header = process_patch_slice(
        &ppf.mmap,
        &mut bin_mmap,
        is_undo,
        on_start,
        progress_callback,
    )?;

    bin_mmap.flush()?;
    Ok(header)
}
