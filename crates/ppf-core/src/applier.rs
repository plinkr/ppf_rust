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

/// Operation mode when modifying binary data with a patch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchAction {
    Apply,
    Undo,
}

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
    PpfFile::open(patch_path).map(|file| file.header)
}

/// Applies or reverts a PPF patch on an in-memory mutable binary slice.
pub fn patch_slice(
    patch_data: &[u8],
    target_data: &mut [u8],
    action: PatchAction,
    on_start: Option<&dyn Fn(usize)>,
    progress_callback: Option<&dyn Fn(usize)>,
) -> Result<PatchInfo, PpfError> {
    let view = PpfView::parse(patch_data)?;
    let is_undo = action == PatchAction::Undo;

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

macro_rules! def_patch_pair {
    (
        $apply_name:ident,
        $undo_name:ident,
        $target_fn:ident,
        $patch_param:ident: $patch_ty:ty,
        $target_param:ident: $target_ty:ty,
        $apply_doc:expr,
        $undo_doc:expr
    ) => {
        #[doc = $apply_doc]
        #[inline]
        pub fn $apply_name(
            $patch_param: $patch_ty,
            $target_param: $target_ty,
            on_start: Option<&dyn Fn(usize)>,
            progress: Option<&dyn Fn(usize)>,
        ) -> Result<PatchInfo, PpfError> {
            $target_fn(
                $patch_param,
                $target_param,
                PatchAction::Apply,
                on_start,
                progress,
            )
        }

        #[doc = $undo_doc]
        #[inline]
        pub fn $undo_name(
            $patch_param: $patch_ty,
            $target_param: $target_ty,
            on_start: Option<&dyn Fn(usize)>,
            progress: Option<&dyn Fn(usize)>,
        ) -> Result<PatchInfo, PpfError> {
            $target_fn(
                $patch_param,
                $target_param,
                PatchAction::Undo,
                on_start,
                progress,
            )
        }
    };
}

def_patch_pair!(
    apply_patch_slice,
    undo_patch_slice,
    patch_slice,
    patch: &[u8],
    target: &mut [u8],
    "Applies a PPF patch (v1.0, v2.0, or v3.0) to an in-memory mutable binary slice.",
    "Reverses/undoes a PPF3 patch on an in-memory mutable binary slice."
);

/// Applies or reverts a PPF patch on a target binary file on disk.
#[cfg(feature = "mmap")]
pub fn patch_file(
    patch_path: impl AsRef<Path>,
    bin_path: impl AsRef<Path>,
    action: PatchAction,
    on_start: Option<&dyn Fn(usize)>,
    progress_callback: Option<&dyn Fn(usize)>,
) -> Result<PatchInfo, PpfError> {
    let ppf = PpfFile::open(patch_path)?;

    let bin_file = OpenOptions::new().read(true).write(true).open(bin_path)?;
    let mut bin_mmap = unsafe { MmapMut::map_mut(&bin_file)? };
    #[cfg(unix)]
    bin_mmap.advise(memmap2::Advice::Sequential)?;

    let header = patch_slice(
        &ppf.mmap,
        &mut bin_mmap,
        action,
        on_start,
        progress_callback,
    )?;

    bin_mmap.flush()?;
    Ok(header)
}

#[cfg(feature = "mmap")]
def_patch_pair!(
    apply_patch,
    undo_patch,
    patch_file,
    patch: impl AsRef<Path>,
    bin: impl AsRef<Path>,
    "Applies a PPF patch (v1.0, v2.0, or v3.0) to a target binary file.",
    "Reverses/undoes a PPF3 patch from a previously patched binary file."
);
