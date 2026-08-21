use thiserror::Error;

/// PPF error types returned across the library.
#[derive(Debug, Error)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PpfError {
    #[error("I/O error: {0}")]
    #[cfg_attr(feature = "serde", serde(skip))]
    Io(#[from] std::io::Error),
    #[error("Invalid magic bytes: {0}")]
    InvalidMagic(String),
    #[error("Corrupted or malformed patch file: {0}")]
    CorruptPatch(String),
    #[error("Bin file size mismatch (expected {expected}, found {actual})")]
    BinSizeMismatch { expected: u64, actual: u64 },
    #[error("Block check validation failed")]
    BlockCheckFailed,
    #[error("Undo data not available in this patch")]
    UndoNotAvailable,
    #[error("Patch offset 0x{offset:X} (len {length}) exceeds target binary length {bin_size}")]
    OffsetOutOfBounds {
        offset: u64,
        length: usize,
        bin_size: u64,
    },
}

/// Supported PPF patch file versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PpfVersion {
    V1,
    V2,
    V3,
}

/// Disc image type for PPF3 validation offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ImageType {
    /// Standard RAW/BIN image (offset 0x9320 for validation).
    Bin,
    /// Global Image / PrimoDVD (offset 0x80A0 for validation).
    Gi,
}

/// Parsed PPF header information.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PpfHeader {
    pub version: PpfVersion,
    pub description: String,
    pub original_bin_length: Option<u32>,
    pub image_type: ImageType,
    pub block_check: bool,
    pub has_undo: bool,
    pub block_data: Option<Vec<u8>>,
    pub file_id: Option<String>,
}

/// Borrowed PPF patch diff record referencing memory-mapped slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PpfRecord<'a> {
    pub offset: u64,
    pub length: u8,
    pub data: &'a [u8],
    pub undo_data: Option<&'a [u8]>,
}

/// Owned PPF patch diff record with allocated Vecs.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PpfPatchRecord {
    pub offset: u64,
    pub length: u8,
    pub data: Vec<u8>,
    pub undo_data: Option<Vec<u8>>,
}

impl<'a> PpfRecord<'a> {
    pub fn to_owned(&self) -> PpfPatchRecord {
        PpfPatchRecord {
            offset: self.offset,
            length: self.length,
            data: self.data.to_vec(),
            undo_data: self.undo_data.map(|u| u.to_vec()),
        }
    }
}
