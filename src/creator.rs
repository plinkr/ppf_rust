use crate::core::{ImageType, PpfError};
use memmap2::Mmap;
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

const WRITE_BUF: usize = 8 * 1024 * 1024;
const CHUNK_SIZE: usize = 1024 * 1024;

/// Configuration options for creating a PPF3 patch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PpfCreatorOptions {
    pub description: String,
    pub image_type: ImageType,
    pub block_check: bool,
    pub undo_data: bool,
    pub file_id: Option<Vec<u8>>,
}

impl Default for PpfCreatorOptions {
    fn default() -> Self {
        Self {
            description: "PPF3 Patch".to_owned(),
            image_type: ImageType::Bin,
            block_check: true,
            undo_data: false,
            file_id: None,
        }
    }
}

#[derive(Clone, Copy)]
struct DiffRange {
    offset: u64,
    start_in_chunk: u32,
    length: u8,
}

/// Creates a PPF3 patch file comparing the original and modified binary files using memory-mapped I/O.
///
/// Returns the number of diff record entries written to the patch file.
///
/// # Arguments
/// * `original_path` - Path to the original unpatched binary file.
/// * `patched_path` - Path to the modified binary file.
/// * `output_path` - Destination path for the resulting PPF3 patch file.
/// * `options` - Configuration options for the PPF3 patch.
/// * `progress_cb` - Optional thread-safe callback invoked with bytes processed in each chunk.
pub fn create_patch(
    original_path: impl AsRef<Path>,
    patched_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    options: &PpfCreatorOptions,
    progress_cb: Option<&(dyn Fn(usize) + Sync + Send)>,
) -> Result<usize, PpfError> {
    let orig_file = File::open(original_path)?;
    let mod_file = File::open(patched_path)?;

    let orig_size = orig_file.metadata()?.len();
    let mod_size = mod_file.metadata()?.len();
    if orig_size != mod_size {
        return Err(PpfError::BinSizeMismatch {
            expected: orig_size,
            actual: mod_size,
        });
    }

    let orig_mmap = unsafe { Mmap::map(&orig_file)? };
    let mod_mmap = unsafe { Mmap::map(&mod_file)? };

    orig_mmap.advise(memmap2::Advice::Sequential)?;
    mod_mmap.advise(memmap2::Advice::Sequential)?;

    let output = BufWriter::with_capacity(WRITE_BUF, File::create(output_path)?);
    create_patch_mmap(&orig_mmap, &mod_mmap, output, options, progress_cb)
}

fn create_patch_mmap<W: Write>(
    orig_data: &[u8],
    mod_data: &[u8],
    mut output: W,
    options: &PpfCreatorOptions,
    progress_cb: Option<&(dyn Fn(usize) + Sync + Send)>,
) -> Result<usize, PpfError> {
    output.write_all(b"PPF30")?;
    output.write_all(&[0x02u8])?;

    let mut desc = [0x20u8; 50];
    let src = options.description.as_bytes();
    let n = usize::min(src.len(), 50);
    desc[..n].copy_from_slice(&src[..n]);
    output.write_all(&desc)?;

    output.write_all(&[
        match options.image_type {
            ImageType::Bin => 0u8,
            ImageType::Gi => 1u8,
        },
        options.block_check as u8,
        options.undo_data as u8,
        0u8,
    ])?;

    if options.block_check {
        let off = match options.image_type {
            ImageType::Bin => 0x9320usize,
            ImageType::Gi => 0x80A0usize,
        };
        let mut block = [0u8; 1024];
        if orig_data.len() > off {
            let available = usize::min(1024, orig_data.len() - off);
            block[..available].copy_from_slice(&orig_data[off..off + available]);
        }
        output.write_all(&block)?;
    }

    let all_diffs: Vec<Vec<DiffRange>> = orig_data
        .par_chunks(CHUNK_SIZE)
        .zip(mod_data.par_chunks(CHUNK_SIZE))
        .enumerate()
        .map(|(chunk_idx, (orig_chunk, mod_chunk))| {
            let base_offset = chunk_idx as u64 * CHUNK_SIZE as u64;
            let diffs = scan_chunk(orig_chunk, mod_chunk, base_offset);
            if let Some(cb) = progress_cb {
                cb(orig_chunk.len());
            }
            diffs
        })
        .collect();

    let mut entries_found = 0;
    let mut header_buf = [0u8; 9];
    let orig_chunks = orig_data.chunks(CHUNK_SIZE);
    let mod_chunks = mod_data.chunks(CHUNK_SIZE);

    for ((orig_chunk, mod_chunk), chunk_diffs) in orig_chunks.zip(mod_chunks).zip(all_diffs) {
        entries_found += chunk_diffs.len();
        for rec in chunk_diffs {
            header_buf[..8].copy_from_slice(&rec.offset.to_le_bytes());
            header_buf[8] = rec.length;
            output.write_all(&header_buf)?;

            let start = rec.start_in_chunk as usize;
            let len = rec.length as usize;
            output.write_all(&mod_chunk[start..start + len])?;
            if options.undo_data {
                output.write_all(&orig_chunk[start..start + len])?;
            }
        }
    }

    if let Some(file_id) = &options.file_id {
        output.write_all(b"@BEGIN_FILE_ID.DIZ")?;
        let len = usize::min(file_id.len(), 3072);
        output.write_all(&file_id[..len])?;
        output.write_all(b"@END_FILE_ID.DIZ")?;
        output.write_all(&(len as u16).to_le_bytes())?;
    }

    Ok(entries_found)
}

#[inline]
fn scan_chunk(orig: &[u8], modif: &[u8], base_offset: u64) -> Vec<DiffRange> {
    let amount = usize::min(orig.len(), modif.len());
    let mut diffs = Vec::new();
    let mut i = 0usize;

    while i < amount {
        while i + 8 <= amount {
            let o = u64::from_ne_bytes(orig[i..i + 8].try_into().unwrap());
            let m = u64::from_ne_bytes(modif[i..i + 8].try_into().unwrap());
            if o != m {
                break;
            }
            i += 8;
        }

        if i < amount && orig[i] == modif[i] {
            i += 1;
            continue;
        }

        if i >= amount {
            break;
        }

        let start = i;
        let offset = base_offset + i as u64;
        let mut k = 0usize;

        while i < amount && orig[i] != modif[i] && k < 255 {
            k += 1;
            i += 1;
        }

        diffs.push(DiffRange {
            offset,
            start_in_chunk: start as u32,
            length: k as u8,
        });
    }

    diffs
}

/// Creates a PPF3 patch from arbitrary `Read + Seek` streams and writes to a `Write` stream.
///
/// Returns the number of diff record entries written to the patch stream.
pub fn create_patch_stream<R1, R2, W>(
    mut original: R1,
    mut modified: R2,
    output: W,
    options: &PpfCreatorOptions,
    progress_cb: Option<&(dyn Fn(usize) + Sync + Send)>,
) -> Result<usize, PpfError>
where
    R1: Read + Seek,
    R2: Read + Seek,
    W: Write,
{
    let orig_size = original.seek(SeekFrom::End(0))?;
    let mod_size = modified.seek(SeekFrom::End(0))?;
    if orig_size != mod_size {
        return Err(PpfError::BinSizeMismatch {
            expected: orig_size,
            actual: mod_size,
        });
    }

    original.seek(SeekFrom::Start(0))?;
    modified.seek(SeekFrom::Start(0))?;

    let mut orig_buf = vec![0u8; orig_size as usize];
    let mut mod_buf = vec![0u8; mod_size as usize];

    original.read_exact(orig_buf.as_mut_slice())?;
    modified.read_exact(mod_buf.as_mut_slice())?;

    create_patch_mmap(
        orig_buf.as_slice(),
        mod_buf.as_slice(),
        output,
        options,
        progress_cb,
    )
}
