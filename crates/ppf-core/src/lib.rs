pub mod applier;
pub mod core;
pub mod creator;
pub mod parser;

pub use applier::{
    PatchAction, PatchInfo, apply_patch_slice, inspect_patch_slice, patch_slice, undo_patch_slice,
};
#[cfg(feature = "mmap")]
pub use applier::{apply_patch, inspect_patch, patch_file, undo_patch};

pub use core::{ImageType, PpfError, PpfHeader, PpfPatchRecord, PpfRecord, PpfVersion};
#[cfg(feature = "mmap")]
pub use creator::create_patch;
pub use creator::{
    DEFAULT_DESCRIPTION, PpfCreatorOptions, create_patch_slice, create_patch_stream,
};

#[cfg(feature = "mmap")]
pub use parser::PpfFile;
pub use parser::{PpfRecords, PpfView};
