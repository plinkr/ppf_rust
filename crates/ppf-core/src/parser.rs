use crate::core::{ImageType, PpfError, PpfHeader, PpfRecord, PpfVersion};
#[cfg(feature = "mmap")]
use memmap2::Mmap;
#[cfg(feature = "mmap")]
use std::fs::File;
#[cfg(feature = "mmap")]
use std::path::Path;

/// Slice-based PPF reader providing zero-copy access to records.
#[derive(Debug, Clone)]
pub struct PpfView<'a> {
    pub header: PpfHeader,
    data: &'a [u8],
    data_start: usize,
    data_end: usize,
}

macro_rules! impl_record_accessors {
    () => {
        #[inline]
        pub fn records(&self) -> PpfRecords<'_> {
            PpfRecords::new(
                self.payload_slice(),
                self.header.version,
                self.header.has_undo,
            )
        }

        #[inline]
        pub fn count_records(&self) -> Result<usize, PpfError> {
            self.records().count_records()
        }
    };
}

impl<'a> PpfView<'a> {
    /// Parses a PPF patch from an in-memory byte slice.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, PpfError> {
        let file_len = bytes.len();

        if file_len < 56 {
            return Err(PpfError::InvalidMagic(
                "File too short to be a valid PPF patch".into(),
            ));
        }

        let version = match &bytes[..4] {
            b"PPF1" => PpfVersion::V1,
            b"PPF2" => PpfVersion::V2,
            b"PPF3" => PpfVersion::V3,
            other => {
                return Err(PpfError::InvalidMagic(
                    String::from_utf8_lossy(other).into_owned(),
                ));
            }
        };

        let desc_bytes = &bytes[6..56];
        let description = String::from_utf8_lossy(desc_bytes)
            .trim_end_matches(['\0', ' '])
            .to_string();

        let mut header = PpfHeader {
            version,
            description,
            original_bin_length: None,
            image_type: ImageType::Bin,
            block_check: false,
            has_undo: false,
            block_data: None,
            file_id: None,
        };

        let mut file_id = None;
        let mut trailer_len = 0usize;

        if version != PpfVersion::V1 {
            let lenidx = if version == PpfVersion::V2 { 4 } else { 2 };
            if file_len >= lenidx + 4 {
                let magic_offset = file_len - (lenidx + 4);
                if &bytes[magic_offset..magic_offset + 4] == b".DIZ" {
                    let len_offset = file_len - lenidx;
                    let id_len = if lenidx == 4 {
                        u32::from_le_bytes(bytes[len_offset..len_offset + 4].try_into().unwrap())
                            as usize
                    } else {
                        u16::from_le_bytes(bytes[len_offset..len_offset + 2].try_into().unwrap())
                            as usize
                    };

                    let fixed_trailer = lenidx + 16 + 18;
                    if file_len >= fixed_trailer + id_len {
                        trailer_len = fixed_trailer + id_len;
                        let text_start = file_len - (lenidx + 16 + id_len);
                        let text_len = usize::min(3072, id_len);
                        let id_bytes = &bytes[text_start..text_start + text_len];
                        file_id = Some(String::from_utf8_lossy(id_bytes).into_owned());
                    }
                }
            }
        }
        header.file_id = file_id;

        let data_start = match version {
            PpfVersion::V1 => 56,
            PpfVersion::V2 => {
                if file_len < 1084 {
                    return Err(PpfError::CorruptPatch(
                        "PPF2 patch file too short for header and block check".into(),
                    ));
                }
                header.original_bin_length =
                    Some(u32::from_le_bytes(bytes[56..60].try_into().unwrap()));
                header.block_check = true;
                header.block_data = Some(bytes[60..1084].to_vec());
                1084
            }
            PpfVersion::V3 => {
                if file_len < 60 {
                    return Err(PpfError::CorruptPatch(
                        "PPF3 patch file too short for header flags".into(),
                    ));
                }
                header.image_type = if bytes[56] == 1 {
                    ImageType::Gi
                } else {
                    ImageType::Bin
                };
                header.block_check = bytes[57] == 1;
                header.has_undo = bytes[58] == 1;

                if header.block_check {
                    if file_len < 1084 {
                        return Err(PpfError::CorruptPatch(
                            "PPF3 patch file too short for block check data".into(),
                        ));
                    }
                    header.block_data = Some(bytes[60..1084].to_vec());
                    1084
                } else {
                    60
                }
            }
        };

        if file_len < data_start + trailer_len {
            return Err(PpfError::CorruptPatch(
                "Patch file length is smaller than header and trailer size".into(),
            ));
        }

        let data_end = file_len - trailer_len;

        Ok(PpfView {
            header,
            data: bytes,
            data_start,
            data_end,
        })
    }

    #[inline]
    pub fn payload_slice(&self) -> &'a [u8] {
        &self.data[self.data_start..self.data_end]
    }

    impl_record_accessors!();
}

/// Memory-mapped PPF file reader providing zero-copy access to records.
#[cfg(feature = "mmap")]
pub struct PpfFile {
    pub header: PpfHeader,
    pub mmap: Mmap,
    data_start: usize,
    data_end: usize,
}

#[cfg(feature = "mmap")]
impl PpfFile {
    /// Opens and parses a PPF patch file using memory-mapped I/O.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, PpfError> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        let (header, data_start, data_end) = {
            let view = PpfView::parse(&mmap)?;
            (view.header, view.data_start, view.data_end)
        };

        Ok(PpfFile {
            header,
            mmap,
            data_start,
            data_end,
        })
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.mmap[..]
    }

    #[inline]
    pub fn payload_slice(&self) -> &[u8] {
        let range = self.data_start..self.data_end;
        &self.mmap[range]
    }

    impl_record_accessors!();
}

pub struct PpfRecords<'a> {
    slice: &'a [u8],
    cursor: usize,
    version: PpfVersion,
    has_undo: bool,
}

impl<'a> PpfRecords<'a> {
    #[inline]
    pub fn new(slice: &'a [u8], version: PpfVersion, has_undo: bool) -> Self {
        Self {
            slice,
            cursor: 0,
            version,
            has_undo,
        }
    }

    pub fn count_records(self) -> Result<usize, PpfError> {
        let mut count = 0;
        for res in self {
            res?;
            count += 1;
        }
        Ok(count)
    }
}

impl<'a> Iterator for PpfRecords<'a> {
    type Item = Result<PpfRecord<'a>, PpfError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let remaining = self.slice.len() - self.cursor;
        if remaining == 0 {
            return None;
        }

        let c = self.cursor;
        let (offset, advance) = match self.version {
            PpfVersion::V1 | PpfVersion::V2 => {
                if remaining < 5 {
                    return Some(Err(PpfError::CorruptPatch(
                        "Truncated record header in PPF patch payload".into(),
                    )));
                }
                let off = u32::from_le_bytes(self.slice[c..c + 4].try_into().unwrap()) as u64;
                (off, 5usize)
            }
            PpfVersion::V3 => {
                if remaining < 9 {
                    return Some(Err(PpfError::CorruptPatch(
                        "Truncated record header in PPF3 patch payload".into(),
                    )));
                }
                let off = u64::from_le_bytes(self.slice[c..c + 8].try_into().unwrap());
                (off, 9usize)
            }
        };

        let length = self.slice[c + advance - 1] as usize;
        let data_start = c + advance;
        let data_end = data_start + length;

        let total_end = if self.has_undo {
            data_end + length
        } else {
            data_end
        };

        if total_end > self.slice.len() {
            return Some(Err(PpfError::CorruptPatch(
                "Patch record length exceeds payload boundary".into(),
            )));
        }

        let data = &self.slice[data_start..data_end];
        let undo_data = if self.has_undo {
            Some(&self.slice[data_end..total_end])
        } else {
            None
        };

        self.cursor = total_end;

        Some(Ok(PpfRecord {
            offset,
            length: length as u8,
            data,
            undo_data,
        }))
    }
}
